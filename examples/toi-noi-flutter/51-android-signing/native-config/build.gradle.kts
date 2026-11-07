import java.util.Properties

// Sao chép thành android/app/build.gradle.kts của app khách Tới Nơi.
// Phần ký bản release (bài 51) nằm giữa hai dòng "Ký bản release".

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// ---- Ký bản release: đọc key.properties (ngoài git) ----
val keystorePropertiesFile = rootProject.file("key.properties")
val keystoreProperties = Properties()
if (keystorePropertiesFile.exists()) {
    keystorePropertiesFile.inputStream().use { keystoreProperties.load(it) }
}
val requiredSigningKeys = listOf("storeFile", "storePassword", "keyAlias", "keyPassword")
// ---- Hết phần đọc key.properties ----

android {
    namespace = "vn.toinoi.customer"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // Application ID theo flavor được thêm ở productFlavors của bài 50.
        applicationId = "vn.toinoi.customer"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        // Lấy từ `version: x.y.z+build` của pubspec.yaml; không gõ số ở đây.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        create("release") {
            // Thiếu key.properties thì các giá trị là null; việc báo lỗi do task
            // verifyReleaseSigning bên dưới đảm nhận, chỉ khi build release.
            keyAlias = keystoreProperties.getProperty("keyAlias")
            keyPassword = keystoreProperties.getProperty("keyPassword")
            storeFile = keystoreProperties.getProperty("storeFile")?.let { file(it) }
            storePassword = keystoreProperties.getProperty("storePassword")
        }
    }

    buildTypes {
        release {
            // Không có nhánh dự phòng sang khoá debug: không ký được thì build hỏng.
            signingConfig = signingConfigs.getByName("release")
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

// ---- Ký bản release: dừng sớm với thông báo rõ ----
val verifyReleaseSigning by tasks.registering {
    // Chép ra biến cục bộ để doLast không giữ tham chiếu tới đối tượng script
    // (cần cho --configuration-cache). Đường dẫn tương đối hiểu từ android/app.
    val propsFile = keystorePropertiesFile
    val props = keystoreProperties
    val requiredKeys = requiredSigningKeys
    val appDir = projectDir
    doLast {
        if (!propsFile.exists()) {
            throw GradleException(
                "Thiếu ${propsFile.path}. Build release cần upload keystore: " +
                    "sao chép key.properties.example thành key.properties và điền giá trị. " +
                    "Bản release không tự ký bằng khoá debug."
            )
        }
        val missing = requiredKeys.filter { props.getProperty(it).isNullOrBlank() }
        if (missing.isNotEmpty()) {
            throw GradleException("key.properties thiếu khoá: ${missing.joinToString(", ")}")
        }
        val storeFile = appDir.resolve(props.getProperty("storeFile"))
        if (!storeFile.exists()) {
            throw GradleException("Không thấy upload keystore tại ${storeFile.path}")
        }
    }
}

// Cổng chạy trước các task pre<Variant>ReleaseBuild (preReleaseBuild, preProdReleaseBuild...)
// nên chặn bản release trước khi ký. Thứ tự với bước biên dịch Dart/Java chưa quan sát được
// trên AGP thật, nên đừng hiểu đây là "chạy trước mọi bước biên dịch".
tasks.configureEach {
    if (Regex("^pre.*ReleaseBuild$").matches(name)) {
        dependsOn(verifyReleaseSigning)
    }
}
// ---- Hết phần ký bản release ----
