require "json"

package = JSON.parse(File.read(File.join(__dir__, "package.json")))

Pod::Spec.new do |s|
  s.name         = "json-eval-rs"
  s.version      = package["version"]
  s.summary      = package["description"]
  s.homepage     = package["homepage"]
  s.license      = package["license"]
  s.authors      = package["author"]

  s.platforms    = { :ios => "12.0" }
  s.source       = { :git => package["repository"]["url"], :tag => "#{s.version}" }

  s.source_files = "ios/**/*.{h,m,mm}", "cpp/**/*.{h,cpp}"
  s.public_header_files = "ios/**/*.h", "cpp/**/*.h"

  s.dependency "React-Core"

  # Rust library paths (bundled with npm package)
  # Uses separate libraries for device and simulator to avoid name conflicts
  # CocoaPods will automatically select the appropriate library based on SDK
  s.ios.vendored_libraries = [
    "ios/libs/libjson_eval_rs_device.a",
    "ios/libs/libjson_eval_rs_simulator.a"
  ]
  
  s.pod_target_xcconfig = {
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17',
    'CLANG_CXX_LIBRARY' => 'libc++',
    'LIBRARY_SEARCH_PATHS' => '$(PODS_TARGET_SRCROOT)/ios/libs',
    # Link the appropriate library based on SDK
    'OTHER_LDFLAGS[sdk=iphonesimulator*]' => '-l"json_eval_rs_simulator"',
    'OTHER_LDFLAGS[sdk=iphoneos*]' => '-l"json_eval_rs_device"'
  }
  
  # System frameworks
  s.frameworks = "Foundation"
  s.libraries = "c++"
end
