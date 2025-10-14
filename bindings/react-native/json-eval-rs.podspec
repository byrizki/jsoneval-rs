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

  # C++ Standard
  s.pod_target_xcconfig = {
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17',
    'CLANG_CXX_LIBRARY' => 'libc++',
    'OTHER_LDFLAGS' => '-force_load $(PODS_TARGET_SRCROOT)/../../target/aarch64-apple-ios/release/libjson_eval_rs.a'
  }

  # Rust library paths
  s.vendored_libraries = "../../target/aarch64-apple-ios/release/libjson_eval_rs.a"
  
  # System frameworks
  s.frameworks = "Foundation"
  s.libraries = "c++"
end
