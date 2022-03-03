package software.amazon.jsii;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.*;

@Internal
final class NativeUtils {
    public static void loadNative(final String name) {
        final String fileName = platformSpecificFileName();
        final String resourcePath = String.format("jni/%s/%s", name, fileName);

        try {
            final File tempFile = Files.createTempFile("jsii-runtime-java", fileName).toFile();

            final ClassLoader cl = NativeUtils.class.getClassLoader();
            try (final InputStream is = cl.getResourceAsStream(resourcePath)) {
                if (is == null) {
                    throw new UnsupportedOperationException(String.format("Unsupported platform: %s", resourcePath));
                }

                Files.copy(is, tempFile.toPath(), StandardCopyOption.REPLACE_EXISTING);
                System.load(tempFile.getAbsolutePath());
            } finally {
                if (isPosixFileSystem()) {
                    // On POSIX platforms, we can delete/unlink right away once the library has been loaded.
                    assert tempFile.delete();
                } else {
                    // On other platforms, we need to wait for the library to no longer be in-use.
                    tempFile.deleteOnExit();
                }
            }
        } catch (final IOException ioe) {
            throw new UncheckedIOException(ioe);
        }
    }

    private static boolean isPosixFileSystem() {
        try {
            return FileSystems.getDefault().supportedFileAttributeViews().contains("posix");
        } catch (final FileSystemNotFoundException | ProviderNotFoundException | SecurityException e) {
            return false;
        }
    }

    private static String platformSpecificFileName() {
        final String osArch = System.getProperty("os.arch");
        final String osName = System.getProperty("os.name");

        if ("Mac OS X".equals(osName)) {
            return String.format("macos-%s.dylib", osArch);
        }

        throw new UnsupportedOperationException(String.format("Unsupported platform: %s %s", osArch, osName));
    }
}