GCC_FLAGS=-std=c++17 -DV8_COMPRESS_POINTERS -DV8_ENABLE_SANDBOX
ifeq ($(DEBUG),1)
	GCC_FLAGS+=-O0 -DV8_ENABLE_CHECKS
else
	GCC_FLAGS+=-O2
endif

build:
	g++ -I./v8include -fPIC -c -g v8_c_api.cpp -o c8_c_api.o $(GCC_FLAGS)
	ar r libv8.a c8_c_api.o
	
clean:
	rm -rf *.o *.a
