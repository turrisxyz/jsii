#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path;

use anyhow::{anyhow, Result};
use jni::objects::{JClass, JList, JMap, JObject, JString};
use jni::JNIEnv;
use jsii_kernel::v8;

lazy_static! {
  static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
}

static mut JAVA_VM: Option<jni::JavaVM> = None;

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelLoad(
  env: JNIEnv,
  _class: JClass,
  name: JString,
  version: JString,
  tarball: JString,
) {
  let name: String = env.get_string(name).unwrap().into();
  let version: String = env.get_string(version).unwrap().into();
  let tarball_path: String = env.get_string(tarball).unwrap().into();
  let tarball_path = path::PathBuf::from(tarball_path);

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelLoad({}, {}, {:?})", &name, &version, &tarball_path);

  match with_kernel(move |kernel| TOKIO_RUNTIME.block_on(kernel.load(&name, &version, &tarball_path))) {
    Ok(_) => return,
    Err(cause) => env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap(),
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelCreate<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  fqn: JString,
  args: JObject,
  _overrides: JObject,
  _interfaces: JObject,
) -> JObject<'a> {
  let fqn: String = env.get_string(fqn).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelCreate({}, ...)", &fqn);

  match with_kernel(move |kernel| {
    let args = {
      let scope = &mut kernel.handle_scope();
      let list = JList::from_env(&env, args);
      if env.exception_check().unwrap() {
        return Err(anyhow!("An exception occurred unpacking the list"));
      }
      list
        .unwrap()
        .iter()
        .unwrap()
        .map(|item| javascript_value(item, env, scope))
        .collect::<Vec<v8::Global<v8::Value>>>()
    };
    if env.exception_check().unwrap() {
      let scope = &mut kernel.handle_scope();
      let null = v8::null(scope);
      let null = null.to_object(scope).unwrap();
      let null = v8::Global::new(scope, null);
      return Ok(null);
    }
    kernel.create(&fqn, args)
  }) {
    Ok(result) => with_kernel(move |kernel| {
      let scope = &mut kernel.handle_scope();
      Ok(java_object(env, scope, result))
    })
    .unwrap(),
    Err(cause) => {
      if !env.exception_check().unwrap() {
        env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
      }
      JObject::null()
    }
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelCall<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  recv: JObject,
  method: JString,
  args: JObject,
) -> JObject<'a> {
  let method: String = env.get_string(method).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelCall(recv, {}, ...)", &method);

  match with_kernel(move |kernel| {
    let (recv, args) = {
      let scope = &mut kernel.handle_scope();
      let recv = javascript_value(recv, env, scope);
      let args = JList::from_env(&env, args);
      if env.exception_check().unwrap() {
        return Err(anyhow!("An exception occurred unpacking the list"));
      }
      let args = args
        .unwrap()
        .iter()
        .unwrap()
        .map(|item| javascript_value(item, env, scope))
        .collect::<Vec<v8::Global<v8::Value>>>();

      (recv, args)
    };
    if env.exception_check().unwrap() {
      let scope = &mut kernel.handle_scope();
      let null: v8::Local<v8::Value> = v8::null(scope).into();
      let null = v8::Global::new(scope, null);
      return Ok(null);
    }
    kernel.call(recv, &method, args)
  }) {
    Ok(result) => with_kernel(move |kernel| {
      let scope = &mut kernel.handle_scope();
      Ok(java_value(env, scope, result))
    })
    .unwrap(),
    Err(cause) => {
      env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
      JObject::null()
    }
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelStaticCall<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  fqn: JString,
  method: JString,
  args: JObject,
) -> JObject<'a> {
  let fqn: String = env.get_string(fqn).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }
  let method: String = env.get_string(method).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelStaticCall({}.{}, ...)", &fqn, &method);

  match with_kernel(move |kernel| {
    let args = {
      let scope = &mut kernel.handle_scope();
      let args = JList::from_env(&env, args);
      if env.exception_check().unwrap() {
        return Err(anyhow!("An exception occurred unpacking the list"));
      }
      args
        .unwrap()
        .iter()
        .unwrap()
        .map(|item| javascript_value(item, env, scope))
        .collect::<Vec<v8::Global<v8::Value>>>()
    };
    if env.exception_check().unwrap() {
      let scope = &mut kernel.handle_scope();
      let null: v8::Local<v8::Value> = v8::null(scope).into();
      let null = v8::Global::new(scope, null);
      return Ok(null);
    }
    kernel.call_static(&fqn, &method, args)
  }) {
    Ok(result) => with_kernel(move |kernel| {
      let scope = &mut kernel.handle_scope();
      Ok(java_value(env, scope, result))
    })
    .unwrap(),
    Err(cause) => {
      env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
      JObject::null()
    }
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelGet<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  recv: JObject,
  property: JString,
) -> JObject<'a> {
  let property: String = env.get_string(property).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelGet(recv, {})", &property);

  match with_kernel(move |kernel| {
    let recv = {
      let scope = &mut kernel.handle_scope();
      javascript_value(recv, env, scope)
    };
    kernel.get(recv, &property)
  }) {
    Ok(result) => with_kernel(move |kernel| {
      let scope = &mut kernel.handle_scope();
      Ok(java_value(env, scope, result))
    })
    .unwrap(),
    Err(cause) => {
      env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
      JObject::null()
    }
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelStaticGet<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  fqn: JString,
  property: JString,
) -> JObject<'a> {
  let fqn: String = env.get_string(fqn).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }
  let property: String = env.get_string(property).unwrap().into();
  if env.exception_check().unwrap() {
    return JObject::null();
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelStaticCall({}.{}, ...)", &fqn, &property);

  match with_kernel(move |kernel| kernel.get_static(&fqn, &property)) {
    Ok(result) => with_kernel(move |kernel| {
      let scope = &mut kernel.handle_scope();
      Ok(java_value(env, scope, result))
    })
    .unwrap(),
    Err(cause) => {
      env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
      JObject::null()
    }
  }
}

#[no_mangle]
pub extern "system" fn Java_software_amazon_jsii_JsiiClient_kernelSet<'a>(
  env: JNIEnv<'a>,
  _class: JClass,
  recv: JObject,
  property: JString,
  value: JObject,
) {
  let property: String = env.get_string(property).unwrap().into();
  if env.exception_check().unwrap() {
    return;
  }

  #[cfg(debug_assertions)]
  eprintln!("[jsii/jni] kernelGet(recv, {})", &property);

  match with_kernel(move |kernel| {
    let (recv, value) = {
      let scope = &mut kernel.handle_scope();
      let recv = javascript_value(recv, env, scope);
      let value = javascript_value(value, env, scope);
      (recv, value)
    };
    kernel.set(recv, &property, value)
  }) {
    Ok(_) => return,
    Err(cause) => {
      env.throw_new("java/lang/RuntimeException", format!("{}", cause)).unwrap();
    }
  }
}

fn javascript_value(item: JObject, env: JNIEnv, scope: &mut v8::HandleScope) -> v8::Global<v8::Value> {
  if env.exception_check().unwrap() {
    // Short-circuit in case an exception was thrown.
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    return v8::Global::new(scope, undefined);
  }

  if item.is_null() {
    let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
    v8::Global::new(scope, undefined)
  } else if env.is_instance_of(item, "software/amazon/jsii/JsiiObject").unwrap() {
    let jsii_object_ref = {
      let val = env.get_field(item, "jsii$objRef", "Lsoftware/amazon/jsii/JsiiObjectRef;");
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      val.unwrap().l().unwrap()
    };

    if jsii_object_ref.is_null() {
      javascript_proxy_value(item, env, scope)
    } else {
      javascript_value(jsii_object_ref, env, scope)
    }
  } else if env.is_instance_of(item, "software/amazon/jsii/JsiiObjectRef").unwrap() {
    let uuid: String = {
      let val = env.call_method(item, "getUuid", "()Ljava/lang/String;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      let val = val.unwrap().l().unwrap();
      env.get_string(val.into()).unwrap().into()
    };

    REVERSE.with(|slot| {
      let map = slot.borrow();
      let global = map.get(&uuid).unwrap();
      let local: v8::Local<v8::Value> = v8::Local::new(scope, global).into();
      v8::Global::new(scope, local)
    })
  } else if env.is_instance_of(item, "java/util/Collections$UnmodifiableList").unwrap() {
    #[cfg(debug_assertions)]
    eprintln!("CHEATING: Un-packing an unmodifiable list (might wrap a JsiiObject-based list proxy)");
    let item = env.get_field(item, "list", "Ljava/util/List;").unwrap().l().unwrap();
    javascript_value(item, env, scope)
  } else if env.is_instance_of(item, "software/amazon/jsii/JsiiSerializable").unwrap() {
    javascript_proxy_value(item, env, scope)
  } else if env.is_instance_of(item, "java/util/List").unwrap() {
    let item = JList::from_env(&env, item).unwrap();
    let size = {
      let size = item.size();
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      size.unwrap()
    };
    let arr = { v8::Array::new(scope, size.try_into().unwrap()) };

    for (i, value) in item.iter().unwrap().enumerate() {
      let value = javascript_value(value, env, scope);
      let value = v8::Local::new(scope, value);
      arr.set_index(scope, i.try_into().unwrap(), value).unwrap();
    }

    let arr: v8::Local<v8::Value> = arr.into();
    v8::Global::new(scope, arr)
  } else if env.is_instance_of(item, "java/util/Map").unwrap() {
    let item = JMap::from_env(&env, item).unwrap();
    let obj = {
      let obj = v8::Object::new(scope);
      let null = v8::null(scope).into();
      obj.set_prototype(scope, null);
      obj
    };

    for (key, value) in item.iter().unwrap() {
      let key = {
        let key: String = env.get_string(key.into()).unwrap().into();
        v8::String::new(scope, &key).unwrap()
      }
      .into();
      let value = {
        let value = javascript_value(value, env, scope);
        v8::Local::new(scope, value)
      };
      obj.set(scope, key, value).unwrap();
    }

    let obj: v8::Local<v8::Value> = obj.into();
    v8::Global::new(scope, obj)
  } else if env.is_instance_of(item, "java/lang/Boolean").unwrap() {
    let item = env.call_method(item, "booleanValue", "()Z", &[]).unwrap().z().unwrap();
    let item: v8::Local<v8::Value> = v8::Boolean::new(scope, item).into();
    v8::Global::new(scope, item)
  } else if env.is_instance_of(item, "java/lang/Number").unwrap() {
    let item = env.call_method(item, "doubleValue", "()D", &[]).unwrap().d().unwrap();
    let item: v8::Local<v8::Value> = v8::Number::new(scope, item).into();
    v8::Global::new(scope, item)
  } else if env.is_instance_of(item, "java/lang/String").unwrap() {
    let item: String = env.get_string(item.into()).unwrap().into();
    let item: v8::Local<v8::Value> = v8::String::new(scope, &item).unwrap().into();
    v8::Global::new(scope, item)
  } else if env.is_instance_of(item, "java/lang/Enum").unwrap() {
    let fqn: String = {
      let class = env.get_object_class(item).unwrap();
      let jsii_class = env.find_class("software/amazon/jsii/Jsii").unwrap().into();
      let ann = env.call_method(
        class,
        "getDeclaredAnnotation",
        "(Ljava/lang/Class;)Ljava/lang/annotation/Annotation;",
        &[jsii_class],
      );
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      let ann = ann.unwrap().l().unwrap();
      let fqn = env.call_method(ann, "fqn", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
      env.get_string(fqn.into()).unwrap().into()
    };
    let name: String = {
      let name = env.call_method(item, "name", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
      env.get_string(name.into()).unwrap().into()
    };
    // HACK: Using the JS API here because we can't get a Kernel in this context... Due to dynamic proxying.
    let func = {
      let source = {
        let code = v8::String::new_external_onebyte_static(scope, b"return global.jsii.get_static(fqn, propertyName);").unwrap();
        v8::script_compiler::Source::new(code, None)
      };
      let fqn = v8::String::new_external_onebyte_static(scope, b"fqn").unwrap();
      let property_name = v8::String::new_external_onebyte_static(scope, b"propertyName").unwrap();
      v8::script_compiler::compile_function_in_context(
        scope,
        source,
        &[fqn, property_name],
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::BecauseInlineScript,
      )
      .unwrap()
    };
    let undefined = v8::undefined(scope).into();
    let fqn = v8::String::new(scope, &fqn).unwrap().into();
    let name = v8::String::new(scope, &name).unwrap().into();
    let result = func.call(scope, undefined, &[fqn, name]).unwrap();
    v8::Global::new(scope, result)
  } else if env
    .is_instance_of(item, "com/fasterxml/jackson/databind/JsonNode")
    .unwrap_or_else(|_| {
      env.exception_clear().unwrap();
      false
    })
  {
    let json = {
      let val = env.call_method(item, "toString", "()Ljava/lang/String;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      val.unwrap().l().unwrap()
    }
    .into();
    let json: String = env.get_string(json).unwrap().into();

    let val = {
      let code = v8::String::new(scope, &format!("({})", &json)).unwrap();
      let source = v8::script_compiler::Source::new(code, None);
      let script = v8::script_compiler::compile(
        scope,
        source,
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::BecauseInlineScript,
      )
      .unwrap();
      script.run(scope).unwrap()
    };

    v8::Global::new(scope, val)
  } else {
    let class_name: String = {
      let class = env.get_object_class(item).unwrap();
      let class_name = env
        .call_method(class, "getCanonicalName", "()Ljava/lang/String;", &[])
        .unwrap()
        .l()
        .unwrap();
      env.get_string(class_name.into()).unwrap().into()
    };
    let item = {
      let val = env.call_method(item, "toString", "()Ljava/lang/String;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      val.unwrap().l().unwrap()
    }
    .into();
    let item: String = env.get_string(item).unwrap().into();
    if !env.exception_check().unwrap() {
      env
        .throw_new(
          "java/lang/UnsupportedOperationException",
          format!("javascript_value({}<{:?}>)", &class_name, item),
        )
        .unwrap();
    }
    let null: v8::Local<v8::Value> = v8::null(scope).into();
    v8::Global::new(scope, null)
  }
}

fn javascript_proxy_value(item: JObject, env: JNIEnv, scope: &mut v8::HandleScope) -> v8::Global<v8::Value> {
  #[cfg(debug_assertions)]
  eprintln!("Creating new anonymous proxy value");

  let obj = {
    let obj = v8::Object::new(scope);
    let null = v8::null(scope).into();
    obj.set_prototype(scope, null);
    obj
  };

  let class = {
    let class = env.get_object_class(item);
    if env.exception_check().unwrap() {
      // Short-circuit in case an exception was thrown.
      let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
      return v8::Global::new(scope, undefined);
    }
    class.unwrap()
  };
  let methods = {
    let methods = env.call_method(class, "getMethods", "()[Ljava/lang/reflect/Method;", &[]);
    if env.exception_check().unwrap() {
      // Short-circuit in case an exception was thrown.
      let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
      return v8::Global::new(scope, undefined);
    }
    methods.unwrap().l().unwrap()
  };

  let count = {
    let count = env.get_array_length(methods.into_inner());
    if env.exception_check().unwrap() {
      // Short-circuit in case an exception was thrown.
      let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
      return v8::Global::new(scope, undefined);
    }
    count.unwrap()
  };
  #[cfg(debug_assertions)]
  eprintln!("Found {} methods to register", count);

  let object_class = {
    let class = env.find_class("java/lang/Object");
    if env.exception_check().unwrap() {
      // Short-circuit in case an exception was thrown.
      let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
      return v8::Global::new(scope, undefined);
    }
    class.unwrap()
  };

  for i in 0..count {
    let method = {
      let method = env.get_object_array_element(methods.into_inner(), i);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      method.unwrap()
    };

    let declaring_class = {
      let declaring_class = env.call_method(method, "getDeclaringClass", "()Ljava/lang/Class;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      declaring_class.unwrap().l().unwrap()
    };
    let is_object_method = {
      let result = env.is_same_object(declaring_class, object_class);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      result.unwrap()
    };
    if is_object_method {
      continue;
    }

    let name: String = {
      let name = env.call_method(method, "getName", "()Ljava/lang/String;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      let name = env.get_string(name.unwrap().l().unwrap().into());
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      name.unwrap().into()
    };

    if name == "equals" || name == "hashCode" || name.starts_with("$") {
      continue;
    }

    let return_type = {
      let return_type = env.call_method(method, "getReturnType", "()Ljava/lang/Class;", &[]);
      if env.exception_check().unwrap() {
        // Short-circuit in case an exception was thrown.
        let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
        return v8::Global::new(scope, undefined);
      }
      let return_type = return_type.unwrap().l().unwrap();

      let return_type = {
        let return_type = env.call_method(return_type, "getCanonicalName", "()Ljava/lang/String;", &[]);
        if env.exception_check().unwrap() {
          // Short-circuit in case an exception was thrown.
          let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
          return v8::Global::new(scope, undefined);
        }
        return_type.unwrap().l().unwrap()
      };

      let return_type = {
        let return_type = env.get_string(return_type.into());
        if env.exception_check().unwrap() {
          // Short-circuit in case an exception was thrown.
          let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
          return v8::Global::new(scope, undefined);
        }
        return_type.unwrap()
      };

      format!("L{};", String::from(return_type).replace(".", "/"))
    };

    unsafe {
      if JAVA_VM.is_none() {
        JAVA_VM = Some(env.get_java_vm().unwrap());
      }
    }

    {
      let is_accessor =
        name.len() > 3 && (name.starts_with("get") || name.starts_with("set")) && name.chars().skip(3).next().unwrap().is_uppercase();
      let java_name = &name;
      if is_accessor {
        let prop_name = format!("{}{}", &name[3..4].to_lowercase(), &name[4..]);
        #[cfg(debug_assertions)]
        eprintln!("Registering property accessor {} for java getter/setter {}", &prop_name, name);
        let name = v8::String::new(scope, &prop_name).unwrap();

        let getter =
          |scope: &mut v8::HandleScope, name: v8::Local<v8::Name>, args: v8::PropertyCallbackArguments, mut ret: v8::ReturnValue| {
            let java = OBJECTS.with(|cell| {
              let this = v8::Global::new(scope, args.this());
              let map = cell.borrow();
              map.get(&this).unwrap().clone()
            });
            let java = java.as_obj();

            let env = unsafe { JAVA_VM.as_ref() }.unwrap().get_env().unwrap();
            let java_method = {
              let this = args.this();
              let name = name.to_string(scope).unwrap();
              let key = v8::Private::for_api(scope, Some(name));
              let value = this.get_private(scope, key).unwrap();
              value.to_rust_string_lossy(scope)
            };
            let (java_name, sig) = java_method.split_once("|").unwrap();

            let result = env.call_method(java, java_name, sig, &[]);
            if env.exception_check().unwrap() {
              // Short-circuit in case an exception was thrown.
              let undefined: v8::Local<v8::Value> = v8::undefined(scope).into();
              ret.set(undefined);
              return;
            }

            let result = javascript_value(result.unwrap().l().unwrap(), env, scope);
            ret.set(v8::Local::new(scope, result));
          };

        {
          let name = v8::Private::for_api(scope, Some(name));
          let sig = format!("(){}", return_type);
          let java_method = v8::String::new(scope, &format!("{}|{}", java_name, sig)).unwrap().into();
          obj.set_private(scope, name, java_method);
        }
        obj.set_accessor(scope, name.into(), getter).unwrap();
      } else {
        #[cfg(debug_assertions)]
        eprintln!("Registering method {}", name);
        let name = v8::String::new(scope, &name).unwrap().into();

        fn method(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut ret: v8::ReturnValue) {
          let message = v8::String::new(scope, "Not implemented").unwrap().into();
          let exception = v8::Exception::error(scope, message);
          scope.throw_exception(exception);

          ret.set(v8::undefined(scope).into());
        }

        let method = v8::FunctionTemplate::builder(method)
          .constructor_behavior(v8::ConstructorBehavior::Throw)
          .side_effect_type(v8::SideEffectType::HasSideEffectToReceiver)
          .build(scope);
        let method = method.get_function(scope).unwrap();
        method.set_name(name);

        obj.set(scope, name.into(), method.into()).unwrap();
      }
    }
    {
      let name = v8::String::new_external_onebyte_static(scope, b"javaObjectID").unwrap();
      let key = v8::Private::for_api(scope, Some(name));
      let uuid = uuid::Uuid::new_v4().to_string();
      let value = v8::String::new(scope, &uuid).unwrap().into();
      obj.set_private(scope, key, value).unwrap();

      OBJECTS.with(|cell| {
        let mut map = cell.borrow_mut();
        let item = env.new_global_ref(item).unwrap();
        let obj = v8::Global::new(scope, obj);
        map.insert(obj, item);
      });
    }
    obj.set_integrity_level(scope, v8::IntegrityLevel::Sealed);
  }

  let obj: v8::Local<v8::Value> = obj.into();
  v8::Global::new(scope, obj)
}

thread_local! {
  static OBJECTS: RefCell<HashMap<v8::Global<v8::Object>, jni::objects::GlobalRef>> = RefCell::new(HashMap::new());
  static REVERSE: RefCell<HashMap<String, v8::Global<v8::Object>>> = RefCell::new(HashMap::new());
}

fn java_object<'java, 'js>(
  env: JNIEnv<'java>,
  scope: &mut v8::HandleScope<'js>,
  value: v8::Global<v8::Object>,
) -> jni::objects::JObject<'java> {
  let local = v8::Local::new(scope, &value);
  if local.is_null_or_undefined() {
    return JObject::null();
  }

  OBJECTS.with(|cell| {
    let mut map = cell.borrow_mut();
    if map.contains_key(&value) {
      let java = map.get(&value).unwrap().clone();
      let java = java.as_obj();
      let java = jni::objects::JObject::<'java>::from(java.into_inner()); // HACK
      env.new_local_ref::<()>(java).unwrap()
    } else {
      let class = {
        let class = env.find_class("software/amazon/jsii/JsiiObjectRef");
        if env.exception_check().unwrap() {
          return JObject::null();
        }
        class.unwrap()
      };
      let uuid = uuid::Uuid::new_v4().to_string();
      let java = {
        let uuid = env.new_string(&uuid).unwrap().into();
        if env.exception_check().unwrap() {
          // Short-circuit in case an exception was thrown.
          return JObject::null();
        }
        let val = env.new_object(class, "(Ljava/lang/String;)V", &[uuid]);
        if env.exception_check().unwrap() {
          // Short-circuit in case an exception was thrown.
          return JObject::null();
        }
        val.unwrap()
      };
      REVERSE.with(|cell| {
        let mut reverse_map = cell.borrow_mut();

        let global = env.new_global_ref(java).unwrap();
        map.insert(value.clone(), global);
        reverse_map.insert(uuid, value);
      });
      java
    }
  })
}

fn java_value<'java, 'js>(
  env: JNIEnv<'java>,
  scope: &mut v8::HandleScope<'js>,
  value: v8::Global<v8::Value>,
) -> jni::objects::JObject<'java> {
  let local = v8::Local::new(scope, value);
  if local.is_null_or_undefined() {
    JObject::null()
  } else if local.is_object() {
    let local = local.to_object(scope).unwrap();
    let value = v8::Global::new(scope, local);
    java_object(env, scope, value)
  } else if local.is_string() {
    let java = env.new_string(local.to_rust_string_lossy(scope));
    if env.exception_check().unwrap() {
      // Short-circuit in case an exception was thrown.
      return JObject::null();
    }
    java.unwrap().into()
  } else if local.is_boolean() {
    let value = local.boolean_value(scope);
    env.new_object("java/lang/Boolean", "(Z)V", &[value.into()]).unwrap()
  } else {
    if !env.exception_check().unwrap() {
      env
        .throw_new(
          "java/lang/UnsupportedOperationException",
          format!("java_value({})", local.to_rust_string_lossy(scope)),
        )
        .unwrap();
    }
    JObject::null()
  }
}

fn with_kernel<T>(cb: impl FnOnce(&mut jsii_kernel::Kernel) -> Result<T>) -> Result<T> {
  thread_local! {
    static KERNEL: RefCell<jsii_kernel::Kernel> = match TOKIO_RUNTIME.block_on(jsii_kernel::Kernel::new()) {
      Ok(kernel) => RefCell::new(kernel),
      Err(cause) => panic!("Failed to initialie jsii kernel: {}", cause),
    };
  }

  KERNEL.with(|cell| {
    let mut kernel = cell.try_borrow_mut()?;
    cb(&mut *kernel)
  })
}
