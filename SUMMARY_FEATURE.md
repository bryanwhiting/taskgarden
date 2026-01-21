# TaskGarden Summary Feature

## Overview
The new `taskgarden summary` command provides flexible task slicing and grouping capabilities to analyze your tasks from different perspectives.

## Usage
```bash
taskgarden summary [OPTIONS]
```

## Options
- `-g, --group <GROUP>` - Group by: date, priority, project, status, context, created (default: date)
- `--created-days <N>` - Show tasks created in the last N days
- `--due-days <N>` - Show tasks due in the next N days  
- `--include-done` - Include completed tasks
- `-s, --sort <SORT>` - Sort groups by: count, time, name (default: name)
- `-d, --detailed` - Show detailed task list for each group

## Examples

### Group by Priority
```bash
taskgarden summary --group priority
```
Shows task counts per priority level and warns if you have too many P0 tasks.

### Group by Project with Details
```bash
taskgarden summary --group project --detailed
```
Shows tasks organized by project with full task details.

### Due Date Analysis
```bash
taskgarden summary --group date --due-days 7
```
Shows tasks due in the next week, grouped by date.

### Sort by Task Count
```bash
taskgarden summary --group project --sort count
```
Shows projects sorted by number of tasks (highest first).

### Include Completed Tasks
```bash
taskgarden summary --group priority --include-done
```
Shows all tasks including completed ones.

## Features
- **Time Calculations**: Automatically sums up time estimates for each group
- **Smart Date Display**: Shows relative dates (today, tomorrow, in X days, etc.)
- **Warnings**: Alerts for overdue tasks or too many high-priority items
- **Flexible Filtering**: Filter by creation date or due date ranges
- **Multiple Sort Options**: Sort by name, count, or total time estimate

## Implementation Notes
- The `created-days` filter requires schema changes to track creation dates (currently not implemented)
- Time estimates are parsed from the task's time field (5m, 15m, 30m, 1h, 2h, 4h, 8h)
- Empty or missing time fields show as "no estimate"