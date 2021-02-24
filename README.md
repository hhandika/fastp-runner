# fastp-runner
[![Build Status](https://www.travis-ci.com/hhandika/fastp-runner.svg?branch=main)](https://www.travis-ci.com/hhandika/fastp-runner)
![fastp-runner](https://github.com/hhandika/fastp-runner/workflows/Tests/badge.svg)

A tool for batch processing NGS data cleaning and adapter trimming using Fastp. The input is a simple csv file. The program allow auto detection.  

# Installation

I will update this soon. In the mean time, check [simple-qc](https://github.com/hhandika/simple-qc) installation instruction. The installation process is similar to it.

Then install fastp. Follow the instruction to install fastp [here](https://github.com/OpenGene/fastp).

# Usage

```
ftr clean -i [csv-input]
```

For dry run to check if the program recognize the right files and adapter sequences.

```
ftr clean -i [csv-input] --dry
```

To check if fastp dependency is installed properly.

```
ftr check
```

It will display the program version if the program can find fastp, such as below:

```
[OK]    fastp 0.20.0
```

# Input File

fastp-runner accept a csv file. The header name is not important. It only needs to have a header file. Otherwise, the program will skip the first line. The order however is important. 

For adapter auto detection:

| id      |
|---------|
|ABCD12345|
|XYZ12456 |

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


# State of the Code
Work in progress. The program is stable. Future update will improve console output and allow for renaming file output. 