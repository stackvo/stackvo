/**
 * Stackvo Documentation - Table Sort Integration
 * Enables sorting functionality for tables
 */

// Wait for DOM and tablesort library to be ready
document.addEventListener('DOMContentLoaded', function() {
    // Check if Tablesort is available
    if (typeof Tablesort !== 'undefined') {
        // Find all tables with class 'sortable' or data-sortable attribute
        const tables = document.querySelectorAll('table.sortable, table[data-sortable]');
        
        tables.forEach(function(table) {
            new Tablesort(table);
        });
        
        console.log('Tablesort initialized for', tables.length, 'tables');
    } else {
        console.warn('Tablesort library not loaded');
    }
});
