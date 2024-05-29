// ../lib/notebook.js
import {invoke} from "@tauri-apps/api/tauri";

export async function deleteNote(noteId, router) {
    if (noteId) {
        try {
            await invoke("delete_note", {id: noteId});
            console.log("Note deleted:", noteId);
            await router.push({name: "Home"});
        } catch (error) {
            console.error("Failed deleting note:", error);
        }
    } else {
        console.log("The note is not defined:", noteId);
    }
}

export async function upsertNote(noteId, noteText) {
    if (noteId) {
        try {
            let note = await invoke("save_note_text", {id: noteId, text: noteText});
            console.log("Note updated:", note.id);
        } catch (error) {
            console.error("Failed saving note:", error);
            // Handle the error as needed, e.g., show a user-friendly message
        }
    } else {
        try {
            let note = await invoke("save_note_text", {id: null, text: noteText});
            console.log("Note created:", note.id);
            return note.id;
        } catch (error) {
            console.error("Failed saving note:", error);
            // Handle the error as needed, e.g., show a user-friendly message
        }
    }
}

export async function getRelatedNotes(noteId, similarityThreshold) {
    try {
        const results = await invoke('get_note_similarities', {id: noteId, threshold: similarityThreshold});
        return results.map(([note, similarityScore]) => ({
            note,
            similarityScore,
        }));
    } catch (error) {
        console.error('Failed to get related notes:', error);
        // Handle the error as needed, e.g., show a user-friendly message
        return [];
    }
}

export function noteTitle(text) {
    const lines = text.split('\n');
    const firstLine = lines[0];
    // remove any leading '#' or spaces
    return firstLine.replace(/^#+\s*/, '');
}