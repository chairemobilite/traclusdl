# TraClus-DL

TraClus-DL analyzes Origin-Destination (OD) data to identify **mobility corridors**. It aggregates individual desire lines (trip lines between an origin and a destination) into corridors, reducing spatial complexity and making dense trajectory datasets easier to visualize and interpret.

The TraClus-DL method is based on the algorithm developed by **Kinnan Bahbouh** in his 2016 thesis: https://publications.polymtl.ca/2433/

---

## How It Works / Key Parameters

Corridor extraction is a clustering process, and the result is **highly sensitive to parameter choice**.

| Parameter Description | What it controls                                                                                          |
| --------------------- | --------------------------------------------------------------------------------------------------------- |
| **Segment size**      | Length into which each desire line is divided before clustering.                                          |
| **Max distance**      | Maximum allowed distance between two segments for them to be considered part of the same corridor.        |
| **Max angle**         | Maximum allowed angular difference between segments, enforcing directional consistency within a corridor. |
| **Min density**       | Minimum number of weighted segments a cluster must contain to be recognized as a corridor.                |

Both **execution time** and **result quality** depend on these four values, so users should expect to tune them for their dataset.

---

## Downloads

There are **two executables per platform**:

| Platform                       | UI executable                | CLI executable                |
| ------------------------------ | ---------------------------- | ----------------------------- |
| Windows (64-bit)               | `traclusdl_ui.exe`           | `traclusdl_cli.exe`           |
| macOS Intel                    | `traclusdl_ui-mac-intel.zip` | `traclusdl_cli-mac-intel.zip` |
| macOS Apple Silicon (M1/M2/M3) | `traclusdl_ui-mac-arm.zip`   | `traclusdl_cli-mac-arm.zip`   |

Download the latest release from the [**GitHub Releases page**](https://github.com/chairemobilite/traclusdl/releases/v1.0.0).

- **UI** (`traclusdl_ui`) : desktop app with a graphical interface to set parameters and run the analysis.
- **CLI** (`traclusdl_cli`) : command-line tool for scripting, automation, or batch runs.

### Windows

1. Download `traclusdl_ui.exe` or `traclusdl_cli.exe`.
2. Double-click to launch.

> If Windows shows a "Windows protected your PC" SmartScreen warning, click **More info** → **Run anyway**.

### macOS

1. Download the `.zip` for your Mac and double-click it to unzip.
2. **Do not double-click the application yet.** Right-click (or Control+click) the app → **Open**.
3. A dialog will appear → click **Open Anyway**.
4. From now on you can double-click it normally.

> If macOS reports that the app is **damaged and can't be opened**:
>
> 1. Open **Terminal** (press ⌘+Space, type `Terminal`, press Enter).
> 2. Type `xattr -cr ` (with a space at the end).
> 3. Drag and drop the application into the Terminal window.
> 4. Press Enter, then try opening the app again.

> **Not sure which Mac you have?** Click the Apple menu () → **About This Mac**.
>
> - If it says **Apple M1 / M2 / M3** → download the `-mac-arm.zip` version.
> - If it says **Intel** → download the `-mac-intel.zip` version.

---

## Building from Source (optional)

**Prebuilt binaries currently only exist for the platforms listed above.** Building from source is therefore the only option for Linux users who want to build the CLI/UI locally, and is otherwise optional.

1. Clone the repository.
2. Install Rust via `cargo`/`rustup`.
3. Build the binaries:

```bash
cargo build --release
cargo build --release --bin traclusdl_ui
cargo build --release --bin traclusdl_cli
```

---

## Running the Application

### Running the UI

```bash
./traclusdl_ui
```

All parameters are set through the interface.

### Running the CLI

Basic usage:

```bash
./traclusdl_cli --file "path/to/od_input_file"
```

From source:

```bash
cargo run --release --bin traclusdl_cli -- --file "path/to/od_input_file"
```

Full argument list:

```bash
./traclusdl_cli --file <filename> --segment_size <segment_size> --max_angle <max_angle> --max_dist <max_dist> --min_density <min_density>
```

Example:

```bash
./traclusdl_cli --file "bigger_sample_traclus.txt" --segment_size 1000 --max_angle 5 --max_dist 600 --min_density 30
```

Other flags:

```text
--interface logger # (Enable verbose/debug logging)
--help #(Show all options, including `--max_threads`, `--mode`, `--interface`, and `--output`.)
```

### Custom Column Mapping (optional)

Use `--map` when the input file's column headers are not in the expected order or when columns need to be mapped explicitly:

```bash
./traclusdl_cli --map "{name:csv_field_name, weight:csv_field_weight, xorigin:csv_field_xorigin, yorigin:csv_field_yorigin, xdest:csv_field_xdest, ydest:csv_field_ydest}"
```

The **keys** must be used exactly as shown:

```text
name, weight, xorigin, yorigin, xdest, ydest
```

The values are the actual column headers from the user's file.

If `--map` is omitted, columns are read in the default order.

---

## Input File Format

### Requirements

| Requirement         | Description                                                                                                          |
| ------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Projection**      | Input coordinates must be in **NAD83 / MTM zone 8**.                                                                 |
| **Units**           | The coordinate unit can be in whatever unit but must match with the unit used for `--max_dist` and `--segment_size`. |
| **File type**       | `.txt` or `.csv`                                                                                                     |
| **Field separator** | Tab, semicolon, or comma                                                                                             |
| **Row format**      | Rows contain 5 or 6 fields (must stay consistent with all rows).                                                     |

### Row Format

| Field      | Type    | Description                           |
| ---------- | ------- | ------------------------------------- |
| `name`     | text    | _(optional)_ Label for the line       |
| `weight`   | integer | Number of trips on this OD line       |
| `x_origin` | decimal | X coordinate of the origin point      |
| `y_origin` | decimal | Y coordinate of the origin point      |
| `x_dest`   | decimal | X coordinate of the destination point |
| `y_dest`   | decimal | Y coordinate of the destination point |

**With `name` (6 fields):**

```text
route_A	3	290424.1	5038577.4	295244.2	5048310.6

```

**Without `name` (5 fields):**

```text
3	290424.1	5038577.4	295244.2	5048310.6
```

A header row is automatically detected and skipped if the first line contains non-numeric values.

Empty lines are ignored.

Sample input files are provided in the [`/data`](./data) folder:

```text
/data/sample_input_traclus_header.txt
/data/sample_input_traclus_no_header.txt
```

---

## Output

The tool produces **two output files**:

| Output        | Description                                          |
| ------------- | ---------------------------------------------------- |
| **Segments**  | Output containing the extracted segments.            |
| **Corridors** | Output containing the identified mobility corridors. |

Both files:

- Are written to the same directory as the executable.
- Use the **same projection and unit as the input file** (NAD83 / MTM zone 8).
- Are intended to be imported into **QGIS** for visualization.

---

## Feedback & Bug Reports

Found a bug or have a suggestion for improvement?

Send an email to [leonard.pouliot@etud.polymtl.ca](mailto:leonard.pouliot@etud.polymtl.ca) with:

- A short description of the bug or idea
- Your platform (Windows / macOS Intel / macOS ARM)
- The input file used, if relevant
