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

/*
Check if file belongs to /dev/dri/, this prevent blocking normal files named renderD128 or card1
*/
static __always_inline int is_dev_dri(struct dentry *dentry){
    struct dentry *parent;
    struct qstr q;
    
    if (!dentry){
        return 1;
    }
    // Check first parent ("dri")
    parent = BPF_CORE_READ(dentry, d_parent);
    if (!parent){
        return 1;
    }
    q = BPF_CORE_READ(parent, d_name);
    if (bpf_strncmp((const char *)q.name, 3, "dri") != 0){
        // not dri
        return 1;
    }
    // Check second parent ("dev")
    parent = BPF_CORE_READ(parent, d_parent);
    if (!parent){
        return 1;
    }
    q = BPF_CORE_READ(parent, d_name);
    if (bpf_strncmp((const char *)q.name, 3, "dev") != 0){
        // not dev
        return 1;
    }
    // Check last parent(root)
    parent = BPF_CORE_READ(parent, d_parent);
    if (!parent){
        return 1;
    }
    q = BPF_CORE_READ(parent, d_name);
    if (bpf_strncmp((const char *)q.name, 1, "/") != 0){
        // not root
        return 1;
    }
    return 0;
}

static __always_inline int get_pci_addr(struct dentry *dentry, char *pci_addr, int size) {
    struct dentry *parent;
    const unsigned char *parent_name;
    int ret;

    if (!dentry) {
        return 1;
    }

    parent = BPF_CORE_READ(dentry, d_parent);
    if (!parent) {
        return 1;
    }
    
    parent_name = BPF_CORE_READ(parent, d_name.name);
    ret = bpf_core_read_str(pci_addr, size, parent_name);
    if (ret < 0) {
        return 1; 
    }
    // Check for PCI address format (eg: 0000:00:00.0.0)
    if (pci_addr[4] == ':' && pci_addr[7] == ':' && pci_addr[10] == '.') {
        return 0;
    }
    
    return 1; 
}

SEC("lsm/file_open")

int BPF_PROG(file_open, struct file *file){
    char filename[16] = {};
    char buf[8] = {};
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
        if (bpf_strncmp(filename, 4, "card") == 0){
            if (is_dev_dri(d) !=0){
                return 0;
            }
            int id = 0;
            int i = 4;
            #pragma unroll
            for (int j = 0; j < 9; j++){
                char c = filename[i];

                if (c >= '0' && c <= '9'){
                    id = id * 10 + (c - '0');
                    i++;
                } else {
                    break;
                }
            }
            int *blocked_id = bpf_map_lookup_elem(&BLOCKED_IDS, &id);
            if (blocked_id){
                return -ENOENT;
            }
        } 
        else if (bpf_strncmp(filename, 7, "renderD") == 0){
            if (is_dev_dri(d) !=0){
                return 0;
            }
            int id = 0;
            int i = 7;
            #pragma unroll
            for (int j = 0; j < 9; j++){
                char c = filename[i];

                if (c >= '0' && c <= '9'){
                    id = id * 10 + (c - '0');
                    i++;
                } else {
                    break;
                }
            }
            u8 *blocked_id = bpf_map_lookup_elem(&BLOCKED_IDS, &id);
            if (blocked_id){
                return -ENOENT;
            }
        }
        else if (bpf_strncmp(filename, 6, "config") == 0){
            char pci_addr[16] = {};
            // check parent path and extract pci to buffer
            if (get_pci_addr(d, pci_addr, sizeof(pci_addr)) != 0){
                return 0;
            }
            pci_addr[11] = '0'; 
            pci_addr[12] = '\0';

            u8 *blocked_pci = bpf_map_lookup_elem(&BLOCKED_PCI, pci_addr);

            if (blocked_pci){
                return -ENOENT;
            }
        }
    }
    return 0;
}