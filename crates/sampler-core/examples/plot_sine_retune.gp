datafile = exists("datafile") ? datafile : "output/sine-retune.csv"
retune_sample = exists("retune_sample") ? retune_sample : 16
outfile = exists("outfile") ? outfile : "output/sine-retune.png"

set datafile separator comma
set terminal pngcairo size 1400,800
set output outfile
set key left top
set grid
set xlabel "Sample index"
set ylabel "Normalized amplitude"
set title "Sine retune comparison"

set style line 1 lc rgb "#4C78A8" lw 2
set style line 2 lc rgb "#E45756" lw 2
set style line 3 lc rgb "#54A24B" lw 2

set arrow 1 from retune_sample, graph 0 to retune_sample, graph 1 nohead dt 2 lc rgb "#777777"

plot datafile using 1:2 with lines ls 1 title "440 Hz baseline", \
     datafile using 1:3 with lines ls 2 title "440 -> 550 Hz retune", \
     datafile using 1:4 with lines ls 3 title "550 Hz fresh"
