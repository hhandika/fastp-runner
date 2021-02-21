# fastp-runner
A tool for batch processing NGS data cleaning and adapter trimming. Take a simple csv file as an input and auto find the read for you.s

Default fastp commands:

Searching is case sensitive
```
fast -i Bunomys_andrewsi_NMVZ21853_S49_L001_R1_001.fastq.gz -I Bunomys_andrewsi_NMVZ21853_S49_L001_R2_001.fastq.gz --adapter_sequence CAAGCAGAAGACGGCATACGAGATGCCATAGGTGACTGGAGTTCAGACGTGT -o clean_read/Bunomys_andrewsi_clean_r1.fastq.gz -O clean_read/bunomys_andrewsi_clean_r2.fastq.gz -q 20
```