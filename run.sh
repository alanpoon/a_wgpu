
cargo apk build --release

set -e

TEMP=$(mktemp -d)

/Users/alanpoon/Documents/softwares/aapt2/aapt2 convert target/release/apk/a_wgpu.apk --output-format proto -o TEMP/app_proto.apk

cd TEMP

unzip app_proto.apk
mkdir manifest

mv AndroidManifest.xml manifest/

rm app_proto.apk
rm -rf META-INF

zip -r base.zip *
zip -jr base.zip *
cd -

java -jar /Users/alanpoon/Documents/softwares/bundletool-all-1.15.6.jar build-bundle --modules=TEMP/base.zip --output=output/android/bundle.aab

bundletool build-bundle --output=output/android/bundle.aab --modules=TEMP/base.zip
--ks=your_keystore.keystore
--ks-pass=ambient
--ks-key-alias=awgpu
--key-pass=ambient

jarsigner -keystore your_keystore.keystore output/android/bundle.aab awgpu