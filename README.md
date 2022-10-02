# Vihr

A simple CLI time usage tracking application.

## Installation

Install using cargo.

```
cargo install --git https://github.com/Windore/vihr.git
```

## Usage

Vihr requires the `VIHR_SAVE_FILE` environment variable to be set up. Set it to any file location that you would like to function as the save location for Vihr's data.

Add a new category:

```
vihr add-category example
```

Start tracking time for the category.

```
vihr start example
```

Start tracking time from a specified point of time. (yyyy-mm-ddThh:mm:ss)

```
vihr start example --start-time 2022-10-2T10:50:00
```

Check if time is currently being tracked.

```
vihr status
```

Stop tracking time.

```
vihr stop 
```

Stop tracking time and specify an optional description for the time spent.

```
vihr stop "Write Vihr README.md"
```

Get a summary for all time spent.

```
vihr summary
```

See all time usage logs for today. For longer logs it is recommended to pipe them to a pager such as `less`.
Logs will be sorted so that the latest time usage is at the top.

```
vihr log today
```

For additional help.

```
vihr help
```

Or

```
vihr help <COMMAND>
```
## License

Copyright (C) 2022 Windore

Vihr is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Vihr is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not,
see <https://www.gnu.org/licenses/>.

See [license](LICENSE).
