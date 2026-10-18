import matplotlib.pyplot as plt, csv, numpy as np, sys, os

save_images = None
use_dark_theme = True

args = iter(sys.argv)
argv0 = next(args, "")
def usage():
    print(f"""\
{argv0} <options...>
available options:
    --help, -h, -? : print this message and exit
    --theme[=]<theme> : colour scheme for the graphs. available themes are "dark" or "light"
    -t : alias for --theme
    --dark, -d : alias for --theme=dark
    --light, -l : alias for --theme=light
    --outdir[=]<outdir> : save the graphs here instead of poping up windows displaying them
    -o : alias for --outdir\
    """)

while arg := next(args, None):
    if len(arg) > 1 and arg[0] == '-':
        key, sep, value = arg.partition("=")
        match key:
            case "--help" | "-h" | "-?": usage(); exit(0)
            case "--theme" | "-t":
                if sep == "": value = next(args, "")
                match value.lower():
                    case "light": use_dark_theme = False
                    case "dark": use_dark_theme = True
                    case _:
                        print(f"invalid value for theme, '{value}', expected dark or light")
                        usage(); exit(2)
            case "--light" | "-l":
                if sep != "": usage(); exit(2)
                use_dark_theme = False
            case "--dark" | "-d":
                if sep != "": usage(); exit(2)
                use_dark_theme = True
            case "--outdir" | "-o":
                if sep == "": value = next(args, "")
                if value == "": usage(); exit(2)
                save_images = value
            case _: print(f"unknown option '{key}'"); usage(); exit(2)
    else:
        print(f"unknown argument '{arg}'"); usage(); exit(2)

data = {}

for signal_name in ["slow sine", "fast sine", "2 sines", "10 sines", "white noise", "sine + white noise"]:
    data[signal_name] = {}
    for type in ["f64 simple", "f32 simple", "f16 simple", "f64 tayor series deg. 30", "f32 tayor series deg. 20", "f16 tayor series deg. 5", "p64 taylor series deg. 30", "p32 taylor series deg. 20", "p16 taylor series deg. 20", "p8 taylor series deg. 20"]:
        with open(f"csv/({type}) ({signal_name}) fft round trip test.csv") as f:
            reader = csv.reader(f)
            header = next(reader)
            res = [[] for _ in range(len(header))]
            for line in reader:
                for i, x in enumerate(line):
                    res[i].append(float(x))
        data[signal_name][type] = {name:col for name,col in zip(header,res)}

# print(data)

if save_images is not None:
    os.makedirs(save_images, exist_ok=True)

if use_dark_theme:
    plt.style.use('dark_background')
    colours = ["orange", "lime", "cyan"]
else:
    colours = ["orange", "green", "blue"]

def do_graph(size, metrics, types, signal_name, log_display=True):
    if isinstance(size, int): size = f"{size} bit"
    plt.figure(figsize=(10,8))
    for type, colour in types:
        for metric, metric_name, line in metrics:
            plt.plot(np.arange(10) + 1, data[signal_name][type][metric], 'o', label=f"{type} {metric_name}", color=colour, ls=line)
    plt.ticklabel_format(style='sci', axis='y', scilimits=(0,0), useMathText=True)
    plt.xticks(np.arange(10) + 1)
    plt.xlabel("n iterations")
    metric_name = metrics[0][1] if len(metrics) == 1 else "error"
    plt.ylabel(metric_name)
    if log_display: plt.yscale('log')
    plt.grid()
    plt.legend()
    plt.title(f"{size} number formats, {metric_name} for {signal_name} after repeated FTT and IFTT")
    plt.tight_layout()
    if save_images is not None:
        plt.savefig(f"{save_images}/{size} {"(log) " if log_display else ""}({metric_name}) ({signal_name}).png")
    # else:
    #     plt.show(block=False)

for signal_name in ["10 sines"]: #, "white noise", "sine + white noise"]:
    # same size comparisons
    for size, degree in [(64, (30, 30)), (32, (20, 20)), (16, (5, 20))]:
        for plot_log in [True, False]:
            do_graph(
                size, 
                [("mae", "mean absolute error", "-"), ("median_err", "median absolute error", "--"), ("p75_err", "75% error", ":"), ("p90_err", "90% error", "-.")],
                [(f"f{size} simple", colours[0]), (f"f{size} tayor series deg. {degree[0]}", colours[1]), (f"p{size} taylor series deg. {degree[1]}", colours[2])],
                signal_name,
                plot_log,
            )
            do_graph(
                size, 
                [("mae", "mean absolute error", "-")],
                [(f"f{size} simple", colours[0]), (f"f{size} tayor series deg. {degree[0]}", colours[1]), (f"p{size} taylor series deg. {degree[1]}", colours[2])],
                signal_name,
                plot_log,
            )
    # different size comparisons
    for size, degree in [((64,32), (30, 20)), ((32,16), (20, 20)), ((16,8), (5, 20))]:
        do_graph(
            f"f{size[0]} and p{size[1]}", 
            [("mae", "mean absolute error", "-"), ("median_err", "median absolute error", "--"), ("p75_err", "75% error", ":"), ("p90_err", "90% error", "-.")],
            [(f"f{size[0]} simple", colours[0]), (f"f{size[0]} tayor series deg. {degree[0]}", colours[1]), (f"p{size[1]} taylor series deg. {degree[1]}", colours[2])],
            signal_name,
            True,
        )
        do_graph(
            f"f{size[0]} and p{size[1]}", 
            [("mae", "mean absolute error", "-")],
            [(f"f{size[0]} simple", colours[0]), (f"f{size[0]} tayor series deg. {degree[0]}", colours[1]), (f"p{size[1]} taylor series deg. {degree[1]}", colours[2])],
            signal_name,
            True,
        )

if save_images is None:
    plts.show()      

# if save_images is None:
#     input()