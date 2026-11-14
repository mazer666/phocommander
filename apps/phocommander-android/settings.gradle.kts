// The settings of the Gradle project of the Android application.
//
// WHY: the project lives in the repository of the whole application and
//       not in a project of its own. The library of Rust is a member of
//       the Cargo workspace, and the two builds meet in the task that
//       copies the library into the package of the application.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "Phocommander"
include(":app")
