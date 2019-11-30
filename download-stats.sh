#!/bin/bash

curl 'http://graphite-kom.srv.lrz.de/render?target=transformNull(sumSeries(exclude(ap.ap*-?mg*.ssid.*,"ssid\\.error$")))&format=json&from=00:00_20190911' > ap-stats.json

