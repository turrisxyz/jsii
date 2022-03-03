package software.amazon.jsii;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.util.Collections;
import java.util.HashSet;
import java.util.Objects;
import java.util.Set;

/**
 * Represents a remote jsii object reference.
 */
@Internal
public final class JsiiObjectRef {
    private final String uuid;

    JsiiObjectRef(final String uuid) {
        this.uuid = uuid;
    }

    public String getUuid() {
        return this.uuid;
    }

    public String toString() {
        return String.format("software.amazon.jsii.JsiiObjectRef<%s>", this.getUuid());
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        JsiiObjectRef that = (JsiiObjectRef) o;
        return Objects.equals(this.uuid, that.uuid);
    }

    @Override
    public int hashCode() {
        return Objects.hash("software.amazon.jsii.JsiiObjectRef", uuid);
    }
}
