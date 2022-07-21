build:
	g++ -c -o2 -g v8_c_api.cpp -o c8_c_api.o -DV8_COMPRESS_POINTERS
	ar r libv8.a c8_c_api.o
	
clean:
	rm -rf *.o *.a