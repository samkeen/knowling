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
            let note = await invoke("save_note", {id: noteId, text: noteText});
            console.log("Note updated:", note.id);
        } catch (error) {
            console.error("Failed saving note:", error);
            // Handle the error as needed, e.g., show a user-friendly message
        }
    } else {
        try {
            let note = await invoke("save_note", {id: null, text: noteText});
            console.log("Note created:", note.id);
            return note.id;
        } catch (error) {
            console.error("Failed saving note:", error);
            // Handle the error as needed, e.g., show a user-friendly message
        }
    }
}