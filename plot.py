#!/usr/bin/env python

import json
import matplotlib.pyplot as pyplot
import os
import pandas
import seaborn

DATA_DIR = 'bench-data'
PLOT_DIR = 'plots'

if not os.path.exists(PLOT_DIR):
    os.mkdir(PLOT_DIR)

seaborn.set_theme()

for filename in os.listdir(DATA_DIR):
    pyplot.clf()
    pyplot.tight_layout()

    path = os.path.join(DATA_DIR, filename)

    with open(path) as f:
        run_info = json.load(f)
    data = pandas.DataFrame(run_info['data'])
    plot = seaborn.barplot(data = data, x = 'implementation', y = 'milliseconds')

    plot.set(
        title = run_info['title'],
        xlabel = 'Implementation',
        ylabel = 'Average runtime (ms)',
    )
    plot.tick_params(axis = 'x', rotation = 30)

    # Add values on top of the bars, including percentage difference for all
    # bars after the first.
    container = plot.containers[0]
    milliseconds = [item['milliseconds'] for item in run_info['data']]
    plot.bar_label(
        container,
        [milliseconds[0]] +
        [f'{y1}\n{((y1 - y0) / y0) * 100:+.2f}%'
            for y0, y1 in zip(milliseconds[:-1], milliseconds[1:])
        ],
    )

    image_file_name = f'{os.path.splitext(filename)[0]}.png'
    plot.get_figure().savefig(
        os.path.join(PLOT_DIR, image_file_name),
        bbox_inches = 'tight',
    )
