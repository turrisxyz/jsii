package software.amazon.jsii;

import com.fasterxml.jackson.databind.JsonNode;
import org.jetbrains.annotations.Nullable;

import java.util.Arrays;

/**
 * A static helper to interact with the kernel in a "simple" way.
 */
@Internal
public final class Kernel {
    /**
     * Calls an async method on the object.
     *
     * @param receiver the receiver for the method call.
     * @param method The name of the method.
     * @param nativeType The return type.
     * @param args Method arguments.
     * @param <T> Java type for the return value.
     *
     * @return A return value.
     */
    @Nullable
    @Internal
    public static <T> T asyncCall(final Object receiver, final String method, final NativeType<T> nativeType, @Nullable final Object... args) {
        throw new UnsupportedOperationException();
    }

    /**
     * Calls a JavaScript method on a receiver.
     *
     * @param receiver the receiver for hte method call
     * @param method The name of the method.
     * @param nativeType The return type.
     * @param args Method arguments.
     * @param <T> Java type for the return value.
     *
     * @return A return value.
     */
    @Nullable
    @Internal
    public static <T> T call(final Object receiver, final String method, final NativeType<T> nativeType, @Nullable final Object... args) {
        final JsiiEngine engine = JsiiEngine.getEngineFor(receiver);
        final JsiiObjectRef objRef = engine.nativeToObjRef(receiver);
        final Object retVal = engine.getClient().callMethod(objRef, method, Arrays.asList(args));
        if (retVal instanceof JsiiObjectRef) {
            return engine.getObject((JsiiObjectRef)retVal, nativeType);
        }
        return nativeType.transform(retVal);
    }

    /**
     * Gets a property value from the object.
     *
     * @param receiver The receiver of the property access.
     * @param property The property name.
     * @param type The Java type of the property.
     * @param <T> The Java type of the property.
     *
     * @return The property value.
     */
    @Nullable
    @Internal
    public static <T> T get(final Object receiver, final String property, final NativeType<T> type) {
        final JsiiEngine engine = JsiiEngine.getEngineFor(receiver);
        final JsiiObjectRef objRef = engine.nativeToObjRef(receiver);

        final Object retVal = engine.getClient().getPropertyValue(objRef, property);
        if (retVal instanceof JsiiObjectRef) {
            return engine.getObject((JsiiObjectRef)retVal, type);
        }
        return type.transform(retVal);
    }

    /**
     * Sets a property value of an object.
     *
     * @param receiver The receiver of the property access.
     * @param property The name of the property.
     * @param value The property value.
     */
    @Internal
    public static void set(final Object receiver, final String property, @Nullable final Object value) {
        final JsiiEngine engine = JsiiEngine.getEngineFor(receiver);
        final JsiiObjectRef objRef = engine.nativeToObjRef(receiver);

        engine.getClient().setPropertyValue(objRef, property, Arrays.asList(value));
    }

    private Kernel() {
        throw new UnsupportedOperationException();
    }
}
