use cardwire_core::gpu::GpuRow;

pub fn print_gpu_table(rows: &[GpuRow]) {
    let mut id_w = 2usize;
    let mut name_w = 4usize;
    let mut pci_w = 3usize;
    let mut render_w = 6usize;
    let default_w = 7usize;
    let blocked_w = 7usize;

    // Calculate widths
    for (id, name, pci, render, _, _) in rows {
        id_w = id_w.max(id.to_string().len());
        name_w = name_w.max(name.len());
        pci_w = pci_w.max(pci.len());
        // Full render string is "renderD" + device number
        let render_full = format!("renderD{}", render);
        render_w = render_w.max(render_full.len());
    }

    // Header
    println!(
        "{:<id_w$}  {:<name_w$}  {:<pci_w$}  {:<render_w$}  {:<default_w$}  {:<blocked_w$}",
        "ID",
        "NAME",
        "PCI",
        "RENDER",
        "DEFAULT",
        "BLOCKED",
        id_w = id_w,
        name_w = name_w,
        pci_w = pci_w,
        render_w = render_w,
        default_w = default_w,
        blocked_w = blocked_w,
    );
    println!(
        "{}  {}  {}  {}  {}  {}",
        "-".repeat(id_w),
        "-".repeat(name_w),
        "-".repeat(pci_w),
        "-".repeat(render_w),
        "-".repeat(default_w),
        "-".repeat(blocked_w),
    );

    for (id, name, pci, render, is_default, blocked) in rows {
        let render_full = format!("renderD{}", render);
        println!(
            "{:<id_w$}  {:<name_w$}  {:<pci_w$}  {:<render_w$}  {:<default_w$}  {:<blocked_w$}",
            id,
            name,
            pci,
            render_full,
            if *is_default { "yes" } else { "no" },
            if *blocked { "on*" } else { "off" },
            id_w = id_w,
            name_w = name_w,
            pci_w = pci_w,
            render_w = render_w,
            default_w = default_w,
            blocked_w = blocked_w,
        );
    }
}
