plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Resolve these once through Monosecret around the release build. Debug
// builds need no secrets; release tasks must never use Android's debug key.
val releaseKeys = listOf(
    "REAPERS_ANDROID_KEYSTORE",
    "REAPERS_ANDROID_STORE_PASSWORD",
    "REAPERS_ANDROID_KEY_ALIAS",
    "REAPERS_ANDROID_KEY_PASSWORD",
)
val signingValues = releaseKeys.associateWith { System.getenv(it) }
val missingSigning = releaseKeys.filter { signingValues[it].isNullOrBlank() }
val releaseRequested = gradle.startParameter.taskNames.any { it.contains("Release") }

if (releaseRequested && missingSigning.isNotEmpty()) {
    throw GradleException(
        "Release signing is not configured. Resolve these through Monosecret: " +
            missingSigning.joinToString(", ")
    )
}

val releaseKeystore = signingValues["REAPERS_ANDROID_KEYSTORE"]?.let { file(it) }
if (releaseRequested && releaseKeystore?.isFile != true) {
    throw GradleException("The configured Reapers release keystore does not exist.")
}

android {
    namespace = "com.reapers.app"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        applicationId = "com.reapers.app"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (missingSigning.isEmpty()) {
            create("reapersRelease") {
                storeFile = releaseKeystore
                storePassword = signingValues["REAPERS_ANDROID_STORE_PASSWORD"]
                keyAlias = signingValues["REAPERS_ANDROID_KEY_ALIAS"]
                keyPassword = signingValues["REAPERS_ANDROID_KEY_PASSWORD"]
            }
        }
    }

    buildTypes {
        release {
            signingConfig = if (missingSigning.isEmpty()) {
                signingConfigs.getByName("reapersRelease")
            } else null
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
