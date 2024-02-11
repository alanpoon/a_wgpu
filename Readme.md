cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build

 adb.arg("shell")
            .arg("pm")
            .arg("list")
            .arg("package")
            .arg("-U")
            .arg(&self.package_name);



RUSTFLAG=INFO cargo run --example hello_world --target=aarch64-apple-darwin