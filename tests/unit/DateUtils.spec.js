// tests/unit/DateUtils.spec.js

import {groupNotesByDate} from '@/lib/DateUtils.js';

describe('DateUtils', () => {
    // ... add test cases for each function

    it('groups notes by date correctly', () => {
        const notes = [
            {id: 1, text: 'Note Now', modified: Date.now() / 1000},
            {id: 2, text: 'Note 1 Day Ago', modified: Date.now() / 1000 - 86400},
            {id: 3, text: 'Note 2 Days Ago', modified: Date.now() / 1000 - 172800},
            {id: 4, text: 'Note 1 Month Ago', modified: Date.now() / 1000 - 2592000},
            {id: 5, text: 'Note 2 Months Ago', modified: Date.now() / 1000 - 5184000},
        ];

        const expectedGroups = {
            'Today': [{id: 1, text: 'Note Now', modified: expect.any(Number)}],
            'Yesterday': [{id: 2, text: 'Note 1 Day Ago', modified: expect.any(Number)}],
            'Earlier this month': [
                {id: 3, text: 'Note 2 Days Ago', modified: expect.any(Number)},
            ],
            'May 2024': [{id: 4, text: 'Note 1 Month Ago', modified: expect.any(Number)}],
            'April 2024': [{id: 5, text: 'Note 2 Months Ago', modified: expect.any(Number)}],
        };

        expect(groupNotesByDate(notes)).toEqual(expectedGroups);
    });
});