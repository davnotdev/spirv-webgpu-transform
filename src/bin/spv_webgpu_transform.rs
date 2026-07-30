use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    let Some((mode, options, input_path, output_path)) = (|| {
        let mode = args.get(1)?;
        let options = args
            .get(2..args.len().checked_sub(2)?)?
            .iter()
            .collect::<Vec<_>>();
        let input_path = args.get(args.len().checked_sub(2)?)?;
        let output_path = args.last()?;
        Some((mode, options, input_path, output_path))
    })() else {
        eprintln!(
            "Usage: spv_webgpu_transform <MODE> [FLAGS] <input.spv> <output.spv>
Modes: combimg|dref|isnanisinf|storagecube|pruneunuseddref|immediates|bindingarray
Flags: 
    --immediates-absolute <N>
    --immediates-max-up-to <N>
    --immediates-max-plus-one-up-to <N>",
        );
        process::exit(1);
    };

    let spv_bytes = fs::read(input_path).unwrap();

    // ------

    let spv = spirv_webgpu_transform::u8_slice_to_u32_vec(&spv_bytes);

    let mut out_correction_map = Default::default();

    let out_spv = match mode.as_str() {
        "combimg" => {
            spirv_webgpu_transform::combimgsampsplitter(&spv, &mut out_correction_map).unwrap()
        }
        "dref" => spirv_webgpu_transform::drefsplitter(&spv, &mut out_correction_map).unwrap(),
        "isnanisinf" => spirv_webgpu_transform::isnanisinfpatch(&spv).unwrap(),
        "storagecube" => {
            spirv_webgpu_transform::storagecubepatch(&spv, &mut out_correction_map).unwrap()
        }
        "pruneunuseddref" => spirv_webgpu_transform::pruneunuseddref(&spv).unwrap(),
        "immediates" => {
            parse_opts(&options, &mut out_correction_map);
            spirv_webgpu_transform::immediatespatch(&spv, &mut out_correction_map).unwrap()
        }
        "bindingarray" => {
            spirv_webgpu_transform::splitbindingarray(&spv, &mut out_correction_map).unwrap()
        }
        mode => {
            eprintln!("unknown mode {:?}", mode);
            process::exit(1)
        }
    };
    let out_spv_bytes = spirv_webgpu_transform::u32_slice_to_u8_vec(&out_spv);

    // ------

    eprintln!("Writing patched result to {}", output_path);
    fs::write(output_path, out_spv_bytes).unwrap();

    if let Some(immediates_set) = out_correction_map.immediates_set {
        println!("Immediates set: {}", immediates_set);
    }

    // Remember to sort your hash maps!
    if let Some(sets) = out_correction_map.sets {
        eprintln!("Finished, patch summary: \n");

        let mut sets = sets.iter().collect::<Vec<_>>();
        sets.sort_by_key(|(k, _)| **k);
        for (set_num, set) in sets {
            println!("Set {}:", set_num);

            let mut bindings = set.bindings.iter().collect::<Vec<_>>();
            bindings.sort_by_key(|(k, _)| **k);
            for (binding_num, binding) in bindings {
                println!("\tBinding {} <- {:?}", binding_num, binding.corrections);
            }
        }
    } else {
        eprintln!("Finished, no correction output sets.");
    }
}

fn get_opt(options: &[&String], name: &str) -> Option<Option<String>> {
    let mut it = options.iter().peekable();

    while let Some(current) = it.next() {
        let next = it.peek();
        if current == &name {
            return Some(next.copied().copied().cloned());
        }
    }
    None
}

fn parse_opts(options: &[&String], correction_map: &mut spirv_webgpu_transform::CorrectionMap) {
    if let Some(Some(n)) = get_opt(options, "--immediates-absolute")
        && let Ok(n) = n.parse::<u32>()
    {
        correction_map.immediates_set = Some(n);
        correction_map.immediates_set_mode =
            Some(spirv_webgpu_transform::ImmediatesSetMode::Absolute);
    }
    if let Some(Some(n)) = get_opt(options, "--immediates-max-up-to")
        && let Ok(n) = n.parse::<u32>()
    {
        correction_map.immediates_set = Some(n);
        correction_map.immediates_set_mode =
            Some(spirv_webgpu_transform::ImmediatesSetMode::MaxUpTo);
    }
    if let Some(Some(n)) = get_opt(options, "--immediates-max-plus-one-up-to")
        && let Ok(n) = n.parse::<u32>()
    {
        correction_map.immediates_set = Some(n);
        correction_map.immediates_set_mode =
            Some(spirv_webgpu_transform::ImmediatesSetMode::MaxPlusOneUpTo);
    }
}
