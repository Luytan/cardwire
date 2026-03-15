#include "vmlinux.h"
#include <bpf/bpf_core_read.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "Dual MIT/GPL";

#define ENOENT 2

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, u32);
    __type(value, u8);
} BLOCKED_IDS SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 1024);
    __type(key, char[16]);
    __type(value, u8);
} BLOCKED_PCI SEC(".maps");


SEC("lsm/file_open")
int BPF_PROG(file_open, struct file *file){
    char filename[256];
    struct dentry *d = BPF_CORE_READ(file, f_path.dentry);
    struct qstr q;
    const unsigned char *name_ptr = NULL;
    int ret;


    if (d){
        q = BPF_CORE_READ(d, d_name);
        name_ptr = q.name;
    }

    if (name_ptr){
        ret = bpf_core_read_str(filename, sizeof(filename), name_ptr);
        if (ret < 0){
            return 0;
        }
    }
    return 0;
}