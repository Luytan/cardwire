#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#ifndef ENOENT
#define ENOENT 2
#endif

typedef unsigned int u32;
typedef unsigned char u8;

struct qstr {
    const unsigned char *name;
} __attribute__((preserve_access_index));

struct dentry {
    struct qstr d_name;
    struct dentry *d_parent;
} __attribute__((preserve_access_index));

struct path {
    struct dentry *dentry;
} __attribute__((preserve_access_index));

struct file {
    struct path f_path;
} __attribute__((preserve_access_index));

char _license[] SEC("license") = "GPL";

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
int BPF_PROG(file_open, struct file *file) {
    struct dentry *dentry = BPF_CORE_READ(file, f_path.dentry);
    const unsigned char *name = BPF_CORE_READ(dentry, d_name.name);

    if (!name) {
        return 0;
    }

    // Read file name prefix.
    char buf[8] = {0};
    bpf_probe_read_kernel_str(&buf, sizeof(buf), name);

    u32 id = 0;
    int is_match = 0;

    // Match renderD<id>.
    if (buf[0] == 'r' && buf[1] == 'e' && buf[2] == 'n' && buf[3] == 'd' && 
        buf[4] == 'e' && buf[5] == 'r' && buf[6] == 'D') {

        // Parse up to 3 digits.
        char id_buf[4] = {0};
        bpf_probe_read_kernel_str(&id_buf, sizeof(id_buf), name + 7);

        #pragma unroll
        for (int i = 0; i < 3; i++) {
            if (id_buf[i] >= '0' && id_buf[i] <= '9') {
                id = id * 10 + (id_buf[i] - '0');
                is_match = 1;
            } else {
                break;
            }
        }
    } 
    // Match card<id>.
    else if (buf[0] == 'c' && buf[1] == 'a' && buf[2] == 'r' && buf[3] == 'd') {
        
        char id_buf[4] = {0};
        bpf_probe_read_kernel_str(&id_buf, sizeof(id_buf), name + 4);

        #pragma unroll
        for (int i = 0; i < 3; i++) {
            if (id_buf[i] >= '0' && id_buf[i] <= '9') {
                id = id * 10 + (id_buf[i] - '0');
                is_match = 1;
            } else {
                break;
            }
        }
    }
    // Match config under PCI device directory.
    else if (buf[0] == 'c' && buf[1] == 'o' && buf[2] == 'n' && buf[3] == 'f' && 
             buf[4] == 'i' && buf[5] == 'g' && buf[6] == '\0') {

        struct dentry *parent = BPF_CORE_READ(dentry, d_parent);
        const unsigned char *parent_name = BPF_CORE_READ(parent, d_name.name);

        if (parent_name) {
            char pci_addr[16] = {0};
            bpf_probe_read_kernel_str(&pci_addr, sizeof(pci_addr), parent_name);

            // Accept only PCI-like parent names: 0000:00:00.0.
            if (pci_addr[4] == ':' && pci_addr[7] == ':' && pci_addr[10] == '.') {
                
                bpf_printk("Checking config for PCI: %s", pci_addr);
                
                // Force the function number to always be '0'
                // This converts "0000:03:00.1" -> "0000:03:00.0"
                pci_addr[11] = '0'; 
                pci_addr[12] = '\0'; 

                // Now look up the base address in the map
                u8 *value = bpf_map_lookup_elem(&BLOCKED_PCI, &pci_addr);
                if (value && *value == 1) {
                    bpf_printk("Blocked config for PCI: %s", pci_addr);
                    return -ENOENT;
                }
            }
        }
    }

    if (is_match) {
        u8 *value = bpf_map_lookup_elem(&BLOCKED_IDS, &id);
        if (value && *value == 1) {
            return -ENOENT;
        }
    }

    return 0;
}