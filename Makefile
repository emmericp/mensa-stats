all: $(patsubst tex/%.tex, img/%.png, $(wildcard tex/*.tex))

ap-stats.json:
	./download-stats.sh

img/%.png: tex/%.pdf
	convert -density 300 -units PixelsPerInch $< $@

tex/%.pdf: tex/%.tex %.csv
	cd tex; pdflatex $(patsubst tex/%.tex, %.tex, $<)

overview.csv: ap-stats.json
	cargo run -- --mode overview --ap-stats $< > $@

average.csv: ap-stats.json
	cargo run -- --mode average --ap-stats $< --since 2019-10-14 --until 2099-01-01 > $@

single-day.csv: ap-stats.json
	cargo run -- --mode single --ap-stats $< --day 2020-01-27 > $@

clean:
	rm -f ap-stats.json
	rm -f tex/*.pdf
	rm -f img/*.png

.PHONY: all clean
