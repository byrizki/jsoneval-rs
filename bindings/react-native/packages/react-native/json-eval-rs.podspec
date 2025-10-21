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
  # Use SDK-specific library search paths to ensure correct library is linked
  s.pod_target_xcconfig = {
    'CLANG_CXX_LANGUAGE_STANDARD' => 'c++17',
    'CLANG_CXX_LIBRARY' => 'libc++',
    # SDK-specific library search paths
    'LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*]' => '$(PODS_TARGET_SRCROOT)/ios/libs/simulator',
    'LIBRARY_SEARCH_PATHS[sdk=iphoneos*]' => '$(PODS_TARGET_SRCROOT)/ios/libs/device',
    # Link the library (name without lib prefix and .a extension)
    'OTHER_LDFLAGS' => '-ljson_eval_rs'
  }
  
  # Prepare library structure before build
  s.prepare_command = <<-CMD
    mkdir -p ios/libs/simulator ios/libs/device
    [ -f ios/libs/libjson_eval_rs_simulator.a ] && cp ios/libs/libjson_eval_rs_simulator.a ios/libs/simulator/libjson_eval_rs.a || true
    [ -f ios/libs/libjson_eval_rs_device.a ] && cp ios/libs/libjson_eval_rs_device.a ios/libs/device/libjson_eval_rs.a || true
  CMD
  
  # System frameworks
  s.frameworks = "Foundation"
  s.libraries = "c++"
end
