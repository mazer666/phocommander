# The rules of the shrinker of the release build.
#
# WHY: the library of Rust is loaded by its name, and the shrinker must
#       not remove the class that loads it. The rules of the Kotlin shell
#       are the default ones of the template of Android.
-keep class dev.phocommander.app.** { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}
