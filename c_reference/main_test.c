#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "rax.h"

int main(void) {
    rax *rt = raxNew();
    assert(rt != NULL);

    char *val1 = "redis_stream_1";
    char *val2 = "redis_cluster_node";
    char *val3 = "redis_command_table";

    // Inserções com chaves com prefixos compartilhados (exige split de nós comprimidos)
    assert(raxInsert(rt, (unsigned char *)"redis:stream:user", 17, val1, NULL) == 1);
    assert(raxInsert(rt, (unsigned char *)"redis:stream:events", 19, val2, NULL) == 1);
    assert(raxInsert(rt, (unsigned char *)"redis:cmd:get", 13, val3, NULL) == 1);

    // Verificações com raxFind
    void *res1 = raxFind(rt, (unsigned char *)"redis:stream:user", 17);
    void *res2 = raxFind(rt, (unsigned char *)"redis:stream:events", 19);
    void *res3 = raxFind(rt, (unsigned char *)"redis:cmd:get", 13);
    void *res4 = raxFind(rt, (unsigned char *)"redis:nonexistent", 17);

    assert(res1 == val1);
    assert(res2 == val2);
    assert(res3 == val3);
    assert(res4 == raxNotFound);

    printf("[Redis Rax Reference] Baseline oficial do Redis executado com sucesso! Elementos: %llu, Nos: %llu\n",
           (unsigned long long)rt->numele, (unsigned long long)rt->numnodes);

    raxFree(rt);
    return 0;
}
