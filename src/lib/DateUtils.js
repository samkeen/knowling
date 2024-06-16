// src/lib/DateUtils.js

/**
 * Checks if two dates are the same day.
 * @param {Date} date1 - The first date to compare.
 * @param {Date} date2 - The second date to compare.
 * @returns {boolean} - Returns true if the dates are the same day, otherwise false.
 */
export function isSameDay(date1, date2) {
    return (
        date1.getFullYear() === date2.getFullYear() &&
        date1.getMonth() === date2.getMonth() &&
        date1.getDate() === date2.getDate()
    );
}

/**
 * Checks if two dates are in the same month.
 * @param {Date} date1 - The first date to compare.
 * @param {Date} date2 - The second date to compare.
 * @returns {boolean} - Returns true if the dates are in the same month, otherwise false.
 */
export function isSameMonth(date1, date2) {
    return (
        date1.getFullYear() === date2.getFullYear() &&
        date1.getMonth() === date2.getMonth()
    );
}

/**
 * Formats a date as a string with the month name and year.
 * @param {Date} date - The date to format.
 * @returns {string} - Returns the formatted date string (e.g., "January 2023").
 */
export function formatMonthYear(date) {
    return date.toLocaleString('default', {month: 'long', year: 'numeric'});
}

/**
 * Formats a date as a number for sorting purposes.
 * @param {Date} date - The date to format.
 * @returns {number} - Returns the formatted date as a number (e.g., 202301).
 */
export function formatMonthYearForSorting(date) {
    return date.getFullYear() * 100 + date.getMonth();
}

/**
 * Groups an array of notes by date.
 * @param {Array} notes - The array of notes to group.
 * @returns {Object} - Returns an object with the grouped notes.
 */
export function groupNotesByDate(notes) {
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    const groups = {
        Today: [],
        Yesterday: [],
        'Earlier this month': [],
    };

    const monthGroups = {};

    notes.forEach(note => {
        const modifiedDate = new Date(note.modified * 1000);

        if (isSameDay(modifiedDate, today)) {
            groups.Today.push(note);
        } else if (isSameDay(modifiedDate, yesterday)) {
            groups.Yesterday.push(note);
        } else if (isSameMonth(modifiedDate, today)) {
            groups['Earlier this month'].push(note);
        } else {
            const monthYear = formatMonthYear(modifiedDate);
            if (!monthGroups[monthYear]) {
                monthGroups[monthYear] = [];
            }
            monthGroups[monthYear].push(note);
        }
    });

    const sortedGroups = {};
    Object.keys(groups).forEach(key => {
        if (groups[key].length > 0) {
            sortedGroups[key] = groups[key];
        }
    });

    const sortedMonthGroups = {};
    Object.keys(monthGroups)
        .sort((a, b) => {
            const dateA = new Date(a.split(' ')[0] + ' 1, ' + a.split(' ')[1]);
            const dateB = new Date(b.split(' ')[0] + ' 1, ' + b.split(' ')[1]);
            return formatMonthYearForSorting(dateB) - formatMonthYearForSorting(dateA);
        })
        .forEach(key => {
            sortedMonthGroups[key] = monthGroups[key];
        });

    return {...sortedGroups, ...sortedMonthGroups};
}