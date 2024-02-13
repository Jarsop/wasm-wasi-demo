if(NOT CMAKE_TOOLCHAIN_FILE)
  message(FATAL_ERROR "CMAKE_TOOLCHAIN_FILE is not set")
endif()

project(AzureCloudProviderWASMModule)

set(SOURCE_FILES src/lib.c)

add_executable(azure_cloud_provider ${SOURCE_FILES})

if(CMAKE_BUILD_TYPE STREQUAL "Release")
  target_compile_options(azure_cloud_provider PRIVATE -O3)
  set(OUTPUT_DIR "release")
else()
  set(OUTPUT_DIR "debug")
endif()

set_target_properties(
  azure_cloud_provider
  PROPERTIES
    LINK_FLAGS
    "-Wl,--export=alloc -Wl,--export=dealloc -Wl,--export=publish -Wl,--no-entry -Wl,--allow-undefined"
)

set_target_properties(
  azure_cloud_provider
  PROPERTIES RUNTIME_OUTPUT_DIRECTORY
             "${CMAKE_BINARY_DIR}/../target/wasm32-wasi/${OUTPUT_DIR}" SUFFIX
                                                                       ".wasm")
