package software.amazon.jsii;

import software.amazon.jsii.api.Callback;
import software.amazon.jsii.api.CreateRequest;
import software.amazon.jsii.api.JsiiOverride;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;

import static software.amazon.jsii.Util.extractResource;

/**
 * HTTP client for jsii-server.
 */
@Internal
public final class JsiiClient {
    private static native void kernelLoad(final String name, final String version, final String tarball);
    private static native JsiiObjectRef kernelCreate(final String fqn, final Collection<Object> initializerArgs, final Collection<JsiiOverride> overrides, final Collection<String> interfaces);
    private static native Object kernelCall(final JsiiObjectRef objRef, final String methodName, final Collection<Object> args);
    private static native Object kernelGet(final JsiiObjectRef objRef, final String property);
    private static native void kernelSet(final JsiiObjectRef objRef, final String property, final Object value);
    private static native Object kernelStaticCall(final String fqn, final String methodName, final Collection<Object> args);
    private static native Object kernelStaticGet(final String fqn, final String property);
    private static native void kernelStaticSet(final String fqn, final String property, final Object value);
    private static native void kernelCompleteCallback(final String cbid, final String error, final Object value);

    private static native void kernelDelete(final JsiiObjectRef objRef);

    static {
        NativeUtils.loadNative("jsii");
    }

    /**
     * JSON node factory.
     */
    private static final JsonNodeFactory JSON = JsonNodeFactory.instance;

    /**
     * Creates a new jsii-runtime client.
     */
    public JsiiClient() {}

    /**
     * Loads a JavaScript module into the remote sandbox.
     * @param module The module to load
     */
    public void loadModule(final JsiiModule module) {
        try {
            Path tarball = extractResource(module.getModuleClass(), module.getBundleResourceName(), null);
            try {
                kernelLoad(module.getModuleName(), module.getModuleVersion(), tarball.toString());
            } finally {
                Files.delete(tarball);
                Files.delete(tarball.getParent());
            }
        } catch (IOException e) {
            throw new JsiiException("Unable to extract resource " + module.getBundleResourceName(), e);
        }
    }

    /**
     * Creates a remote jsii object.
     * @param fqn The fully-qualified-name of the class.
     * @param initializerArgs Constructor arguments.
     * @param overrides A list of async methods to override. If a method is defined as an override, a callback
     *                  will be scheduled when it is called, and the promise it returns will only be fulfilled
     *                  when the callback is completed.
     * @return A jsii object reference.
     */
    public JsiiObjectRef createObject(final String fqn,
                                      final Collection<Object> initializerArgs,
                                      final Collection<JsiiOverride> overrides,
                                      final Collection<String> interfaces) {
        return kernelCreate(fqn, initializerArgs, overrides, interfaces);
    }

    /**
     * Deletes a remote object.
     * @param objRef The object reference.
     */
    public void deleteObject(final JsiiObjectRef objRef) {
        kernelDelete(objRef);
    }

    /**
     * Gets a value for a property from a remote object.
     * @param objRef The remote object reference.
     * @param property The property name.
     * @return The value of the property.
     */
    @SuppressWarnings("unchecked")
    public <T> T getPropertyValue(final JsiiObjectRef objRef, final String property) {
        return (T)kernelGet(objRef, property);
    }

    /**
     * Sets a value for a property in a remote object.
     * @param objRef The remote object reference.
     * @param property The name of the property.
     * @param value The new property value.
     */
    public void setPropertyValue(final JsiiObjectRef objRef, final String property, final Object value) {
        kernelSet(objRef, property, value);
    }

    /**
     * Gets a value of a static property.
     * @param fqn The FQN of the class
     * @param property The name of the static property
     * @return The value of the static property
     */
    @SuppressWarnings("unchecked")
    public Object getStaticPropertyValue(final String fqn, final String property) {
        return kernelStaticGet(fqn, property);
    }

    /**
     * Sets the value of a mutable static property.
     * @param fqn The FQN of the class
     * @param property The property name
     * @param value The new value
     */
    public void setStaticPropertyValue(final String fqn, final String property, final Object value) {
        kernelStaticSet(fqn, property, value);
    }

    /**
     * Invokes a static method.
     * @param fqn The FQN of the class.
     * @param method The method name.
     * @param args The method arguments.
     * @return The return value.
     */
    @SuppressWarnings("unchecked")
    public Object callStaticMethod(final String fqn, final String method, final Collection<Object> args) {
        return kernelStaticCall(fqn, method, args);
    }

    /**
     * Calls a method on a remote object.
     * @param objRef The remote object reference.
     * @param method The name of the method.
     * @param args Method arguments.
     * @return The return value of the method.
     */
    public Object callMethod(final JsiiObjectRef objRef, final String method, final Collection<Object> args) {
        return kernelCall(objRef, method, args);
    }

    /**
     * Begins the execution of an async method.
     * @param objRef The object reference.
     * @param method The name of the async method.
     * @param args Arguments for the method.
     * @return A {@link JsiiPromise} which represents this method.
     */
    public JsiiPromise beginAsyncMethod(final JsiiObjectRef objRef, final String method, final ArrayNode args) {
        throw new UnsupportedOperationException("Unsupported");
    }

    /**
     * Ends the execution of an async method.
     * @param promise The promise returned by beginAsyncMethod.
     * @return The method return value.
     */
    public JsonNode endAsyncMethod(final JsiiPromise promise) {
        throw new UnsupportedOperationException("Unsupported");
    }

    /**
     * Dequques all the currently pending callbacks.
     * @return A list of all pending callbacks.
     */
    public List<Callback> pendingCallbacks() {
        throw new UnsupportedOperationException("Unsupported");
    }

    /**
     * Completes a callback.
     * @param callback The callback to complete.
     * @param error Error information (or null).
     * @param result Result (or null).
     */
    public void completeCallback(final Callback callback, final String error, final JsonNode result) {
        kernelCompleteCallback(callback.getCbid(), error, result);
    }

    /**
     * Returns all names for a jsii module.
     * @param moduleName The name of the module.
     * @return The result (map from "lang" to language configuration).
     */
    public JsonNode getModuleNames(final String moduleName) {
        throw new UnsupportedOperationException("unsupported");
    }
}
