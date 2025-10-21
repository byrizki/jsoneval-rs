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
  # Use on_demand_resources to provide SDK-specific libraries
  s.ios.vendored_library = 'ios/libs/libjson_eval_rs_device.a'
  
  s.pod_target_xcconfig = {
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17',
    'CLANG_CXX_LIBRARY' => 'libc++',
    'LIBRARY_SEARCH_PATHS' => '$(PODS_TARGET_SRCROOT)/ios/libs',
    'OTHER_LDFLAGS[sdk=iphonesimulator*]' => '-force_load "$(PODS_TARGET_SRCROOT)/ios/libs/libjson_eval_rs_simulator.a"',
    'OTHER_LDFLAGS[sdk=iphoneos*]' => '-force_load "$(PODS_TARGET_SRCROOT)/ios/libs/libjson_eval_rs_device.a"'
  }
  
  # System frameworks
  s.frameworks = "Foundation"
  s.libraries = "c++"
end
