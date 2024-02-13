#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum { GET = 0, POST = 1, PUT = 2, DELETE = 3 } e_method;

extern int fetch(e_method method, void *url_ptr, int url_len, void *body_ptr,
                 int body_len);

void *alloc(size_t len) { return (unsigned char *)malloc(len); }

void dealloc(void *ptr, size_t len) {
  (void)len;
  free(ptr);
}

int publish(void *payload, size_t len) {
  e_method method = POST;

  unsigned char *body = (unsigned char *)malloc(len + 20);
  memcpy(body, payload, len);
  strcat((char *)body, " from C WASM [AZURE]");

  const char *url = "http://127.0.0.1:3000/echo";
  int url_len = strlen(url);

  int result =
      fetch(method, (void *)url, url_len, (void *)body, strlen((char *)body));

  printf("WASM: fetch: %d\n", result);
  printf("WASM: payload: %s\n", (char *)payload);
  printf("WASM: TOKEN: %s\n", getenv("TOKEN"));

  free(body);

  return 0;
}
