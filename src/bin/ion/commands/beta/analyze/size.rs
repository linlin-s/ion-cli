use std::fs::File;
use crate::commands::{IonCliCommand, WithIonCliArgument};
use anyhow::{bail, Context, Result};
use clap::{ArgMatches, Command};
use ion_rs::{IonReader, RawBinaryReader, SystemReader, SystemStreamItem};
use memmap::MmapOptions;
use lowcharts::plot;


pub struct SizeCommand;

impl IonCliCommand for SizeCommand {
    fn name(&self) -> &'static str {
        "size"
    }

    fn about(&self) -> &'static str {
        "Prints the overall min, max, mean size of top-level values in the input stream."
    }

    fn configure_args(&self, command: Command) -> Command {
        command.with_input()
    }

    fn run(&self, _command_path: &mut Vec<String>, args: &ArgMatches) -> Result<()> {
        if let Some(input_file_names) = args.get_many::<String>("input") {
            for input_file in input_file_names {
                let file = File::open(input_file.as_str())
                    .with_context(|| format!("Could not open file '{}'", &input_file))?;
                let mmap = unsafe {
                    MmapOptions::new()
                        .map(&file)
                        .with_context(|| format!("Could not mmap '{}'", input_file))?
                };
                // Treat the mmap as a byte array.
                let ion_data: &[u8] = &mmap[..];
                let raw_reader = RawBinaryReader::new(ion_data);
                let mut system_reader = SystemReader::new(raw_reader);
                size_analyze(&mut system_reader);
            }
        } else {
            bail!("this command does not yet support reading from STDIN")
        }
        Ok(())
    }
}

fn size_analyze(reader: &mut SystemReader<RawBinaryReader<&[u8]>>) -> Result<()> {
    let mut vec: Vec<f64> = Vec::new();
    loop {
        match reader.next()? {
            SystemStreamItem::Value(_) => {
                let mut size = 0;
                if reader.annotations_length() != None {
                    size = reader.annotations_length().unwrap() +  reader.header_length() + reader.value_length();
                } else {
                    size = reader.header_length() + reader.value_length();
                }
                vec.push(size as f64);
            },
            SystemStreamItem::Nothing => break,
            _ => {}
        }
    }
// Plot a histogram of the above vector, with 4 buckets and a precision
// chosen by library
    let options = plot::HistogramOptions { intervals: 4, ..Default::default() };
    let histogram = plot::Histogram::new(&vec, options);
    print!("{}", histogram);
    Ok(())
}
