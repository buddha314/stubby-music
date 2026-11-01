## ADDED Requirements

### Requirement: Complete filenames on hover
The Project asset browser SHALL show the complete filesystem filename, including
its extension, when the user hovers an asset tile or list row, including its
thumbnail or label. It SHALL reuse Jackdaw's existing tooltip timing, theme,
placement, wrapping and dismissal conventions and avoid duplicate filename popovers.

#### Scenario: Hovering a truncated gallery entry
- **WHEN** the user hovers the thumbnail or label of a gallery entry whose displayed filename is shortened
- **THEN** the standard tooltip reveals the entire filename and extension after the existing hover delay

#### Scenario: Long Unicode filename
- **WHEN** a long filename containing Unicode characters is revealed in a tooltip
- **THEN** the complete text remains readable through wrapping within the editor window, without truncation or overlapping duplicate filename tooltips

### Requirement: Gallery and list presentation choice
The Project asset browser SHALL offer Gallery and List modes through its existing
themed toolbar conventions, with Gallery as the initial default and an evident
selected mode. The mode SHALL remain selected during the editor session.

#### Scenario: Selecting list mode
- **WHEN** the user selects List in the browser toolbar
- **THEN** the current entries appear as detailed rows and the toolbar indicates List is selected

#### Scenario: Returning to gallery
- **WHEN** the user selects Gallery after using List
- **THEN** the existing thumbnail gallery presentation returns without importing assets again

#### Scenario: Navigating while using list mode
- **WHEN** the user navigates to another folder or closes and reopens the Project panel during the same editor session
- **THEN** the browser retains the selected presentation mode

### Requirement: Detailed list contents
List mode SHALL show each entry with a thumbnail or existing fallback icon, its
full filename including extension, and its filesystem last-modified date/time.
It SHALL label the columns Thumbnail, Name and Last modified and keep full names
readable without ellipsis, wrapping them as necessary. Missing timestamps SHALL
show an unavailable-value marker instead of a fabricated date.

#### Scenario: Viewing a model or prefab
- **WHEN** a model or prefab appears in List mode
- **THEN** its row shows the existing cached thumbnail or fallback icon, its complete filename, and its last-modified date/time with a clear timezone convention

#### Scenario: A filename exceeds the name column
- **WHEN** the complete filename is wider than the available name column
- **THEN** it wraps into a readable full name while keeping the date column readable

#### Scenario: Modification time cannot be read
- **WHEN** an entry's filesystem modification time is unavailable
- **THEN** the row shows an em dash for Last modified and the browser remains usable

### Requirement: One browser model and thumbnail service
Both presentations SHALL share the current folder, entry ordering, search, kind
filter, selection, existing file interactions and thumbnail service. Switching
views SHALL preserve that state. Modification metadata SHALL be refreshed through
the existing scan/refresh lifecycle rather than read for every row every frame.

#### Scenario: Switching with filters and a selected asset
- **WHEN** the user switches views while a folder, search, kind filter and selected asset are active
- **THEN** those values and the inspector subject remain unchanged, preserving visibility of the selected item where possible

#### Scenario: Using an asset from a list row
- **WHEN** the user opens, renames, drags or invokes a context action on a list entry
- **THEN** it follows the existing Project browser interaction semantics

#### Scenario: A file changes on disk
- **WHEN** the existing filesystem refresh detects an entry was modified
- **THEN** its cached modification metadata and existing thumbnail invalidation behavior update in either view without per-frame metadata polling
