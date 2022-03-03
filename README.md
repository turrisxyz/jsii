# jsii.rs

## Overview

`jsii.rs` is an experimental fork of the `jsii` runtime that uses [🦕 Deno][deno] to execute the
JavaScript code that backs the library. Currently it comes with partial :coffee: Java bindings.

The new runtime architecture this leverages is the following:

```plain
┌────────────────────────────────────────────────────────┐
│                                                        │
│                    Host Application                    │
│                                                        │
│    ┌───────────────────────────────────────────────────┤
│    │                                                   │
│    │                Generated Bindings                 │
│    │                                                   │
│    ├───────────────────────────────────────────────────┤
│    │                                                   │
│    │             Host jsii Runtime Library             │
│    │                                                   │
├────┴───────────────────────────────────────────────────┤
│                       ┌────────────────────────────────┤
│                       │                                │
│                       │ Interop. Layer (JNI, FFI, ...) │
│     Host Runtime      │                                │
│ (JVM, .NET Core, ...) │  ┌─────────────────────────────┤
│                       │  │                             │
│                       │  │        Embedded Deno        │
│                       │  │ ┌───────────────────────────┤
│                       │  │ │        Embedded V8        │
├───────────────────────┴──┴─┴───────────────────────────┤
│                                                        │
│                    Operating System                    │
│                                                        │
└────────────────────────────────────────────────────────┘
```

[deno]: https://deno.land

## Embedding Deno

I considered different layers for embedding:

1. `deno_core`
2. `deno_runtime`
3. `deno` (the CLI)

Options 1 and 2 turned out to not be satisfactory, as so much of the logic needed to obtain a
complete runtime with Node compatibility lives in the Deno CLI package: the CommonJS module loader
and NodeJS compatibility shims, permissions engine, etc...

Unfortuately, the `deno` crate is *not* a library crate, and so it cannot be used to embed the full
Deno runtime... The next best thing being embedding `deno_runtime` and reproducing the `deno`
functionality that is needed -- a non-trivial endeavor.

Instead of doing this, I've decided to basically fork `deno` and turn it into a library crate, so I
could minimally modify it to export just the elements I need in order to get my deed done:

* Some convenience re-exports:
  - `deno_core`
  - `deno_runtime`
  - `deno_core::v8`
* Exported modules
  - `compat` provides the necessary gear to load CommonJS and install NodeJS compatibility shims
* Exported types
  - `crate::flags::Flags`
  - `crate::flags::CheckFlag`
  - `crate::flags::RunFlags`
  - `crate::flags::DenoSubcommand`
  - `deno_runtime::permissions::Permissions`
  - `crate::flags::ProcState`
* Exported members
  - `create_main_worker` is used to create a fully configured `deno_runtime::MainWorker` instance,
    complete with NodeJS compatibility and permission boundaries

Effectively, this reproduces the behavior of `deno run` except it hard-codes the CLI flags, and does
not return immediately after having executed the `main_module`, instead continues to interact with
the managed `v8::Isolate`.

A relatively small JavaScript library is loaded (as the `main_module`), which creates a
`global.jsii` object which exposes the API that Java bindings will leverage through JNI. Those are
modeled closely with the current `@jsii/kernel` API (`load`, `create`, `call`, `get`, etc...) with
similar request structures, but does away with much of the internal state management, which is made
redundant thanks to the in-process nature of this architecture.

## What this makes possible

### Reduced Performance Overhead

The in-process architecture would allow broad simplification to the Java runtime code and reduces
the marshalling toll between the two languages. This proof-of-concept runtime has similar
performance to the current subprocess-based runtime, however it was built with minimal changes to
the Java runtime library's architecture, which was designed around a subprocess-based architecture,
and could be significantly streamlined (reducing many expensive Java reflection operations).
Additionally, the proof-of-concept JNI implementation does not perform any caching of classes,
field and method IDs, etc... Adding caching is an easy avenue to improve the overall performance of
the JNI bindings (this is in fact part of the recommendations when developing JNI bindings).

This means the performance of a productionized version of this proof-of-concept can be expected to
be better than that of the subprocess-based architecture. This is not to mention that the in-process
architecture removes the overhead of a distinct process, as well as the IPC overhead.

### Better Memory Management

Through this architecture, Java and JavaScript can directly interact with each other, which removes
the need for serialization and de-serialization (via JSON). This results in faster calls with much
less overhead, and a reduced risk for bugs caused by the lossy nature of the conversion to JSON.

This is also friendlier to returning anonymous object instances from JavaScript to Java without
having to arbitrarily determine whether this will be passed by-value (as a JSON object) or
by-reference, which allows for more flexible downtsream usage.

### Direct access to collections

While this requires a lot of work in Java, moving the JavaScript execution in-process reduces the
penalty involved when making calls between JavaScript & Java. This means it is possible to offer
implementations of `java.util.List` and `java.util.Map` that have decent performance, allowing the
Java code to make mutations that reflect on the JavaScript view of the object (and vice-versa).

This should however not be a design goal: class libraries should instead be designed in a way that
does not rely on arbitrary mutations on collections, as these make it nearly impossible to uphold
invariants. All APIs should expose only reaodnly collections, and have dedicated APIs to perform
mutations on the underlying collections while maintaining invariants.

## Opportunities & Challenges

### Deno embedder API

As one can guess, there is an opportunity to design an embedder API for Deno that includes all the
features exposed by the Deno CLI. This would significantly ease the task for developers who are
looking to use Deno for their in-process JavaScript execution needs. Such an API is however
notoriously difficult to design: it needs to have an abstraction level that provides a clean and
simple mental model, without removing the flexibility users might need in order to tailor the
runtime to their particular needs.

### `v8` objects are not `Send`

When writing the JNI bindings to expose the functionality to Java, the fact that Deno's runtime is
strictly single-threaded (all the way to the `v8` crate, see [denoland/rusty_v8#738]) is currently a
source of complexity: it makes it impossible to hold a `static` instance of the `v8::Isolate` that
the JNI layer needs to deal with, as this implies a requirement that `v8::Isolate` be `Send`.

I could work around this issue by using `thread_local!` as the current jsii runtimes do not support
multi-thread operations at this stage. Moving the JavaScript execution in-process would however make
it relatively easy to support multi-thread client applications, when this limitation of `v8` gets
removed.

On the other hand, the fact `v8::Global` is not `Send` also prevents from attaching `v8::Global`
handles to Java objects using the JNI API (`jni::JNIEnv::set_rust_field`,
`jni::JNIEnv::get_rust_field`, `jni::JNIEnv::take_rust_field`). Instead of this, the library must
keep an external mapping of relationships between the Java and JavaScript views of an object.

[denoland/rusty_v8#738]: https://github.com/denoland/rusty_v8/pull/738

### `jni::objects::GlobalRef` is neither `Hash` nor `Eq`

This might be a relatively easy fix, but the current implementation of `jni::objects::GlobalRef`
lacks implementations for the `Hash` and `Eq` trait, making them unusable as `HashMap` keys. This
adds a little complexity to the internal state tracking, as a `String` value has to be used as a
stand-in for the Java object. A V4 UUID is used for this purpose, and is attached as a private
property of the Java placeholder object.

### Missing support for Weak References

In order to fully benefit from running JavaScript in-process, the internal state traking should use
weak references (both on V8 and Java objects), so that these can be garbage collected as
appropriate. The `v8` crate does not yet expose the necessary V8 APIs. An open pull request aims to
address this gap already: [denoland/rusty_v8#895]. The `jni` crate lacks any specific binding to
this effect, although the `jni-sys` crate (that `jni` is built on) features the necessary APIs to
build user-land support for these.

Additionally, `v8` does not expose `v8::Isolate::asjust_amount_of_external_allocated_memory`, which
could otherwise be used to help V8 determine when to perform garbage collection. Further down, the
ability to use a custom allocator with `v8` would allow the `v8::Isolate` to return the courtesy
to the Java VM (the [proof-of-concept] demonstrated that the `node` heap fills up at a significantly
higher rate than the Java VM's does, which results in the Java Garbage Collector not running
frequently enough to allow JavaScript memory to be made releasable).

Of course, should these features be implemented, the library will still need to track the original
ownership of objects, so it can determine when strong references can be demoted to weak references.
This has been conceptually demonstrated by a [proof-of-concept] I made during last year's hackathon.

[denoland/rusty_v8#895]: https://github.com/denoland/rusty_v8/pull/895
[proof-of-concept]: https://github.com/aws/jsii/tree/rmuller/v2/memory-management

### Using the `jni` API is extremely verbose

Most calls to the `jni::JNIEnv` methods are susceptible to cause a Java exception to be thrown. The
native code operating under JNI does not share Java's approach to dealing with exceptions, and as
such any thrown exception is deferred until the current `Thread` control returns to the JVM. The
honus is on the developer to constantly verify that no exception is pending before performing JNI
calls (as most of them are illegal to make if an exception is pending). This forces developers to
sprinkle `jni::exception_check` calls everywhere, with early return provisions. This incurs a lot
of boilerplate.

Additionally, the JNI architecture uses both local and global references to track objects that are
currently visible to the native code running under JNI. While the model is conceptually similar to
that of `v8`, the implementation currently suffers from excessively restrictive lifecycle
declarations (a fix is being introduced in [jni-rs/jni-rs#319]).

Generally speaking, the `jni` crate offers relatively low-level bindings on top of `jni-sys` that
make integrating with JNI easier, however there is an opportunity to design higher-level APIs that
offer a more idiomatic experience in rust.

[jni-rs/jni-rs#319]: https://github.com/jni-rs/jni-rs/pull/319

### The performance of `jni` can be poor

Best practices when writing JNI libraries insist on caching at least method and field IDs to avoid
expensive lookups. Additionally, class instances can be cached to speed up `instanceof` checks
performed by the bindings layer.

No caching has been done in this proof-of-concept, as this would otherwise force usage of the lower
level API, which is less usable than the APIs used in the current code... It would not have been
possible to achieve a working proof-of-concept within the hackathon's timeline if that additional
layer of indirection had been maintained.

### Bug in CDK

In several places, CDK uses JavaScript splats (`{...props}`)  as a way to merge user-provided
configuration properties into default values. However, those behave in suprising ways when a user
explicitly sets any property to `undefined`: this intends to explicitly request the default value
to be retained, but this is not what happens in practice:

```ts
const userProvided = { foo: undefined };

{ foo: 'bar', ...userProvided };
//(ACTUAL  )=> { foo: undefined }
//(EXPECTED)=> { foo: 'bar' }
```

In particular, generated `metric<Name>` functions use this idiom, looking like:

```ts
public metricError(opts: MetricOptions): Metric {
  return new Metric({ statistic: 'Sum', ...opts });
}
```

In effect, this means providing an `undefined` statistic explicitly in `opts` will cause the default
`statistic` value `'Average'` to be used instead of `'Sum'`, resulting in unintended configuration.

This is problematic because property bags are passed from Java to JavaScript via the creation of a
JavaScript proxy object, where all properties are present, implemented as dynamic accessors that
reach back to Java in order to obtain the value. This behaves identically to specifying an object
literal will all properties present (including explicit `undefined` values).

This is the cause of some behavioral difference between the in-process runtime and the
subprocess-based runtime, which passes property bags by-value instead. While it would be possible to
make the in-process runtime behave identically to the current runtime, this would intorduce
unnecessary complexity (and marshaling costs), and would not address the user experience issue this
exposed.
