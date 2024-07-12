cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build

 adb.arg("shell")
            .arg("pm")
            .arg("list")
            .arg("package")
            .arg("-U")
            .arg(&self.package_name);



RUSTFLAG=INFO cargo run --example hello_world --target=aarch64-apple-darwin
keytool -genkey -v -keystore your_keystore.keystore -alias awgpu -keyalg RSA -keysize 2048 -validity 10000


jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore your_keystore.keystore ./target/release/a_wgpu-unaligned.apk awgpu

java -jar pepk.jar --keystore=your_keystore.keystore --alias=awgpu --output=output.zip --include-cert --rsa-aes-encryption --encryption-key-path=encryption_public_key.pem

java -jar pepk.jar --keystore=deployment_cert-2.der --alias=awgpu --output=output.zip --include-cert --rsa-aes-encryption --encryption-key-path=encryption_public_key.pem

$ java -jar pepk.jar --keystore=your_keystore.keystore --alias=awgpu --output=output.zip --include-cert --rsa-aes-encryption --encryption-key-path=encryption_public_key.pem

password: ambient

cargo run --target aarch64-apple-darwin --release run ../guest/rust/examples/basics/primitives --quic-interface-port 4433

cargo run --target aarch64-apple-darwin join eu.proxy.ambient.run:9167

adb push /Users/alanpoon/Documents/rust/ambient_stuffs/a_wgpu/target/release/apk/a_wgpu.apk /sdcard/Download

adb pull /sdcard/Download/a_wgpu.aab /Users/alanpoon/Documents/rust/ambient_stuffs/a_wgpu/target/release/apk