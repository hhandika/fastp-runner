# fastp-runner

[![Build Status](https://www.travis-ci.com/hhandika/fastp-runner.svg?branch=main)](https://www.travis-ci.com/hhandika/fastp-runner)
![fastp-runner](https://github.com/hhandika/fastp-runner/workflows/Tests/badge.svg)

A tool for batch processing NGS data cleaning and adapter trimming using Fastp. The input is a simple csv file. The program allow auto detection.  

## Quick Start

`fastp-runner` is a single executable application. See the installation instruction [below](#installation) for more details. You will also need [fastp](https://github.com/OpenGene/fastp) to use fastp-runner. After you install fastp-runner and fastp, check to see whether fastp-runner can detect fastp installation.

```{Bash}
ftr check

```

It will show the fastp version installed in your computer if the program can detect it.

```{Bash}
[OK]    fastp 0.20.0
```

For read cleaning and adapter trimming, the folder structure is as below:

```{Bash}
raw_reads/
├── Bulimus_bagobus_ABCD12345_R1.fastq.gz
├── Bulimus_bagobus_ABCD12345_R2.fastq.gz
├── Bunomys_andrewsi_CDEF1245_R1.fastq.gz
├── Bunomys_andrewsi_CDEF1245_R2.fastq.gz
├── Bunomys_andrewsi_XYZ12345_R1.fastq.gz
├── Bunomys_andrewsi_XYZ12345_R2.fastq.gz
└── config.csv
```

All your raw reads files should be in the same folder, including the config file. For most user, the config file is a one-column csv file that contains your sequence names. More input options are available [see below](#input-file). Fastp will auto detect the adapter sequences in your reads. For example above, our config file will be as below:

| samples                   |
|---------------------------|
|Bulimus_bagobus_ABCD12345  |
|Bunomys_andrewsi_CDEF1245  |
|Bunomys_andrewsi_XYZ12345  |

After preparing the config file, you can do dry-run to check if the program accurately detects your sequencing reads.

```{Bash}
ftr clean -i raw_reads/config.csv --dry
```

To process your reads, the command is as below:

```{Bash}
ftr clean -i raw_reads/config.csv
```

You can also use new name for the clean read outputs. The input file will be as below:

| samples                   | new_names                             |
|---------------------------|---------------------------------------|
|Bulimus_bagobus_ABCD12345  | Bulimus_bagobus_ABCD12345_leyte       |
|Bunomys_andrewsi_CDEF1245  | Bunomys_andrewsi_CDEF1245_sulawesi    |
|Bunomys_andrewsi_XYZ12345  | Bunomys_andrewsi_XYZ12345_sulawesi    |

You need to pass the flag `--rename` to rename the files. The command will be as below:

```{Bash}
ftr clean -i raw_reads/config.csv --rename
```

The program folder structure follows [phyluce](https://phyluce.readthedocs.io/en/latest/) pipeline folder structure for full compatibility with the program. Following our example, the final folder structure is as below:

```{Bash}
.
├── clean_reads
│   ├── Bulimus_bagobus_ABCD12345
│   │   ├── fastp_reports
│   │   ├── raw_read_symlinks
│   │   └── trimmed_reads
│   ├── Bunomys_andrewsi_CDEF1245
│   │   ├── fastp_reports
│   │   ├── raw_read_symlinks
│   │   └── trimmed_reads
│   └── Bunomys_andrewsi_XYZ12345
│       ├── fastp_reports
│       ├── raw_read_symlinks
│       └── trimmed_reads
└── raw_reads
```

Your cleaned reads are saved in the `trimmed_reads` folder for each sample.

```{Bash}
trimmed_reads/
├── Bulimus_bagobus_ABCD12345_R1.fastq.gz
└── Bulimus_bagobus_ABCD12345_R2.fastq.gz
```

The fastp_reports consist of three files:

```{Bash}
fastp_reports/
├── fastp.html
├── fastp.json
└── fastp.log
```

The html and json files contain similar information about sequence quality before and after cleaning for human to view and for machine to process, respectively. A fastp html sample output can be found [here](http://opengene.org/fastp/fastp.html).

The log file is terminal output that you would see if you run fastp directly. `fastp-runner` removes this ouput from terminal to reduce clutter and redirect it to a file. If there is an error when fastp process the file, fastp-runner will also display fastp ouput in the terminal for your convenient.

## Installation

I will update this soon. In the mean time, check [simple-qc](https://github.com/hhandika/simple-qc) installation instruction. The installation process is similar to it.

Then install fastp. Follow the instruction [here](https://github.com/OpenGene/fastp).

## Usage

```{Bash}
ftr clean -i [csv-input]
```

For dry run to check if the program recognize the right files and adapter sequences.

```{Bash}
ftr clean -i [csv-input] --dry
```

To check if fastp dependency is installed properly.

```{Bash}
ftr check
```

It will display the program version if the program can find fastp, such as below:

```{Bash}
[OK]    fastp 0.20.0
```

## Input File

fastp-runner accept a csv file. The header name is not important. It only needs to have a header file. Otherwise, the program will skip the first line. The order however is important.

If the first three part of your filename is your sample name, such as genus_epithet_museum#_read.fastq.gz and you would like to keep the same name for the raw-reads and the cleaned reads. You can pass the unique id of your sample, such as museum numbers. The config file will be as below:

| id      |
|---------|
|ABCD12345|
|XYZ12456 |

You can also pass your adapter sequence. In our test, there is no data quality differences between letting fastp detects the adapter versus providing the adapter sequences in the config file. Having adapter sequences in the config file only helps the program to run slightly faster. To provide the adapter sequences, the config file will be as below. If you use the sequence name, replace the IDs with the sequence names.

For cvs file with a single adapter:
| id        |   i7                |
| -------   | ------------------- |
|XYZ12345   | ATGTCTCTCTATATATACT |
|ABC12345   | ATGTCTCTCTATATATACT |

For csv file with dual adapters:

| id      |   i5                | i7                  |
| ------- | ------------------- | ------------------  |
|XYZ12345 | ATGTCTCTCTATATATACT | ATGTCTCTCTATATATGCT |
|ABC12345 | ATGTCTCTCTATATATACT | ATGTCTCTCTATATATGCT |

If the adapter and tag were splitted:

The program will automatically insert the tag. It follows the algorithms used by [Illumiprocessor](https://illumiprocessor.readthedocs.io/en/latest/).

| id      |   i5                    | i7                    |   i5 index   | i7 index   |
| -------   | -------------------   | ------------------    | -----------| ---------|
|XYZ12345    | ATGTCTCTCTATATATAC*T | ATGTCTCTCTATATATGC*T  | ATGTCTC    | ATGTATG  |
|ABC12345    | ATGTCTCTCTATATATAC*T | ATGTCTCTCTATATATGC*T  | GGGTCTC    | ATGTAAA  |

## Commands

Sub-commands available for fastp-runner:

```{Bash}
USAGE:
    ftr <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check    Checks if fastp is installed
    clean    Runs fastp
    help     Prints this message or the help of the given subcommand(s)
```

For data cleaning and adapter trimming:

```{Bash}
USAGE:
    ftr clean [FLAGS] [OPTIONS]

FLAGS:
        --dry        Checks if the program detect the correct files
    -h, --help       Prints help information
        --id         Uses id instead of filenames
        --rename     Renames output files
    -V, --version    Prints version information

OPTIONS:
    -i, --input <INPUT>    Inputs a config file
```

## State of the Code

Work in progress. The program is stable. Future update will improve console output and allow for renaming file output.
