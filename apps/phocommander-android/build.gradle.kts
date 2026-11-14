// The Gradle project of the Android application of Phocommander.
//
// WHAT: the build of the application: the manifest of the package, the
//       library of Rust that the application loads, and the copy of that
//       library into the package of the application.
// WHY: the Rust core does the work and the Kotlin shell does the access.
//       The shell therefore needs the library of Rust, and the library is
//       built for the ABI of the device — four of them, because a file
//       manager in the store runs on every device of the last years.
//
// The build of the library of Rust needs the Android NDK, and the build of
// the NDK is not part of this repository: the NDK is installed by the
// `ANDROID_NDK_HOME` of the machine that builds. The task below names the
// directory of the NDK and the target of the build, so that the error of a
// missing NDK says what is missing.

plugins {
    id("com.android.application") version "8.7.3"
    id("org.jetbrains.kotlin.android") version "2.0.21"
}

android {
    namespace = "dev.phocommander.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "dev.phocommander.app"
        // WHY: the application runs on Android 8 and later. The version
        //       that the storage of the system protects is the one that
        //       makes the SAF mode necessary.
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    // WHY: the four ABIs of the devices of the last years. A file manager
    //       that does not run on a device of the user is of no use to him.
    ndk {
        abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    // WHY: the library of Rust is built by Cargo and not by Gradle. The
    //       task below does it and puts the result where the packaging of
    //       the application expects it.
    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
}

sourceSets {
    getByName("main") {
        java.srcDirs("src/main/kotlin")
    }
}

// The libraries of Rust for the four ABIs of the device.
val rustTargets = mapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86_64" to "x86_64-linux-android",
    "x86" to "i686-linux-android",
)

/**
 * Builds the library of Rust for one ABI.
 *
 * WHY: the rules of the documents are in the Rust core, and the core is
 *       compiled for the device. The task names the target of the build
 *       and the directory of the NDK, so that the error of a missing NDK
 *       says what is missing.
 */
fun buildRust(target: String, abi: String) {
    val ndk = System.getenv("ANDROID_NDK_HOME")
        ?: error(
            "ANDROID_NDK_HOME is not set. The library of Rust needs the " +
                "Android NDK; see docs/acceptance/M4-ACCEPTANCE.md.",
        )
    val cargo = rootProject.file("../..").resolve("scripts/cargo.sh")
    exec {
        workingDir = rootProject.file("../..")
        commandLine(
            cargo.absolutePath,
            "build",
            "--release",
            "--target", target,
            "-p", "phocommander-android",
        )
        environment("ANDROID_NDK_HOME", ndk)
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", "$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang")
        environment("CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER", "$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang")
        environment("CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER", "$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang")
        environment("CARGO_TARGET_I686_LINUX_ANDROID_LINKER", "$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android21-clang")
    }
    copy {
        from(rootProject.file("../../target/$target/release/libphocommander_android.so"))
        into(layout.buildDirectory.dir("rustJniLibs/$abi"))
    }
}

tasks.register("buildRustLibrary") {
    description = "Builds the library of Rust for the ABIs of the device."
    group = "build"
    rustTargets.forEach { (abi, target) ->
        doLast { buildRust(target, abi) }
    }
}

tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn("buildRustLibrary")
}
