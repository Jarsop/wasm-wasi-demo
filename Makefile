WASI_SDK_VERSION = 21
WASI_SDK_FULL_VERSION = $(WASI_SDK_VERSION).0
WASI_SDK_URL = https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$(WASI_SDK_VERSION)/wasi-sdk-$(WASI_SDK_FULL_VERSION)-linux.tar.gz
WASI_SDK_PATH = wasi-sdk

DEBUG ?=
ifeq ($(DEBUG), 1)
	CMAKE_BUILD_TYPE = Debug
	DIR_SUFFIX = debug
else
	CARGO_BUILD_FLAGS = --release
	CMAKE_BUILD_TYPE = Release
	DIR_SUFFIX = release
endif

WASM_TARGET_DIR = target/wasm32-wasi/$(DIR_SUFFIX)
HOST_TARGET_DIR = target/$(DIR_SUFFIX)

AWS_CLOUD_PROVIDER = $(WASM_TARGET_DIR)/aws_cloud_provider.wasm
AWS_CLOUD_PROVIDER_SRC = cloud-provider/aws/src/lib.rs
AZURE_CLOUD_PROVIDER = $(WASM_TARGET_DIR)/azure_cloud_provider.wasm
AZURE_CLOUD_PROVIDER_SRC = cloud-provider/azure/src/lib.c

CLOUD_CONSUMER = $(HOST_TARGET_DIR)/cloud-consumer
CLOUD_CONSUMER_SRC = cloud-consumer/src/main.rs
ECHO_SERVER = $(HOST_TARGET_DIR)/echo-server
ECHO_SERVER_SRC = echo-server/src/main.rs

.PHONY: all clean

all: $(AWS_CLOUD_PROVIDER) $(AZURE_CLOUD_PROVIDER) $(CLOUD_CONSUMER) $(ECHO_SERVER)
	@echo All targets have been built

$(WASI_SDK_PATH):
	@echo Downloading WASI SDK $(WASI_SDK_FULL_VERSION)...
	@curl -sL $(WASI_SDK_URL) | tar xz
	@mv wasi-sdk-$(WASI_SDK_FULL_VERSION) $(WASI_SDK_PATH)
	@echo WASI SDK $(WASI_SDK_FULL_VERSION) has been downloaded and extracted to $(WASI_SDK_PATH)

$(AWS_CLOUD_PROVIDER): $(AWS_CLOUD_PROVIDER_SRC)
	@echo Building AWS Cloud Provider...
	@cargo -q build $(CARGO_BUILD_FLAGS) --target wasm32-wasi -p aws-cloud-provider
	@echo AWS Cloud Provider has been built

$(AZURE_CLOUD_PROVIDER): $(WASI_SDK_PATH) $(AZURE_CLOUD_PROVIDER_SRC)
	@echo Building Azure Cloud Provider...
	@cmake -B build -S . -DCMAKE_TOOLCHAIN_FILE=$(WASI_SDK_PATH)/share/cmake/wasi-sdk.cmake -DCMAKE_BUILD_TYPE=$(CMAKE_BUILD_TYPE) >/dev/null 2>&1
	@cmake --build build >/dev/null 2>&1
	@echo Azure Cloud Provider has been built

$(CLOUD_CONSUMER): $(CLOUD_CONSUMER_SRC)
	@echo Building Cloud Consumer...
	@cargo -q build $(CARGO_BUILD_FLAGS) -p cloud-consumer
	@echo Cloud Consumer has been built

$(ECHO_SERVER): $(ECHO_SERVER_SRC)
	@echo Building Echo Server...
	@cargo -q build $(CARGO_BUILD_FLAGS) -p echo-server
	@echo Echo Server has been built

clean:
	@rm -rf build
	@cargo -q clean

clean-all: clean
	@rm -rf $(WASI_SDK_PATH)
