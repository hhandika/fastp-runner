# fastp-runner
A tool for batch processing NGS data cleaning and adapter trimming using Fastp. The input is a simple csv file. The program allow auto detection.  

Work in progress...

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