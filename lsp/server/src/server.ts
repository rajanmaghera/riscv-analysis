/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
import {
	createConnection,
	TextDocuments,
	Diagnostic,
	DiagnosticSeverity,
	ProposedFeatures,
	InitializeParams,
	DidChangeConfigurationNotification,
	CompletionItem,
	CompletionItemKind,
	TextDocumentPositionParams,
	TextDocumentSyncKind,
	InitializeResult,
	WorkspaceFolder
} from 'vscode-languageserver/node';

import {
	TextDocument
} from 'vscode-languageserver-textdocument';

const rust = import('../pkg');

// Create a connection for the server, using Node's IPC as a transport.
// Also include all preview / proposed LSP features.
const connection = createConnection(ProposedFeatures.all);

// Create a simple text document manager.
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;

const diagnosticsSentTo: string[] = []; // a list of uris whose diagnostics have been sent to the client. We want to clear diagnostics for files that are not open anymore


connection.onInitialize((params: InitializeParams) => {



	const capabilities = params.capabilities;

	// Does the client support the `workspace/configuration` request?
	// If not, we fall back using global settings.
	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	);
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	);
	hasDiagnosticRelatedInformationCapability = !!(
		capabilities.textDocument &&
		capabilities.textDocument.publishDiagnostics &&
		capabilities.textDocument.publishDiagnostics.relatedInformation
	);

	const result: InitializeResult = {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Incremental,
			// Tell the client that this server supports code completion.
			completionProvider: {
				resolveProvider: true
			}
		}
	};
	if (hasWorkspaceFolderCapability) {
		result.capabilities.workspace = {
			workspaceFolders: {
				supported: true
			}
		};
	}
	return result;
});

connection.onInitialized(() => {
	if (hasConfigurationCapability) {
		// Register for all configuration changes.
		connection.client.register(DidChangeConfigurationNotification.type, undefined);
	}
	if (hasWorkspaceFolderCapability) {


		// connection.workspace.getWorkspaceFolders().then(folders => {

		// 	workspaceFolders = folders;
		// 	connection.console.log("Folders: " + JSON.stringify(folders));

		// 	if (folders) {
		// 		connection.console.log('Workspace folder change event received.');
		// 	}

		// });

		connection.workspace.onDidChangeWorkspaceFolders(_event => {
			connection.console.log('Workspace folder change event received.');
		});
	}
});

// The example settings
interface ExampleSettings {
	maxNumberOfProblems: number;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: ExampleSettings = { maxNumberOfProblems: 1000 };
let globalSettings: ExampleSettings = defaultSettings;

// Cache the settings of all open documents
const documentSettings: Map<string, Thenable<ExampleSettings>> = new Map();

connection.onDidChangeConfiguration(change => {
	if (hasConfigurationCapability) {
		// Reset all cached document settings
		documentSettings.clear();
	} else {
		globalSettings = <ExampleSettings>(
			(change.settings.languageServerExample || defaultSettings)
		);
	}

	// Revalidate all open text documents
	validateAllTextDocuments();
	// documents.all().forEach(validateTextDocument);
});

function getDocumentSettings(resource: string): Thenable<ExampleSettings> {
	if (!hasConfigurationCapability) {
		return Promise.resolve(globalSettings);
	}
	let result = documentSettings.get(resource);
	if (!result) {
		result = connection.workspace.getConfiguration({
			scopeUri: resource,
			section: 'languageServerExample'
		});
		documentSettings.set(resource, result);
	}
	return result;
}


// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidChangeContent(change => {
	validateAllTextDocuments();
});


documents.onDidOpen(change => {
	connection.console.log("Server opened file: " + change.document.uri);
	validateAllTextDocuments();
});

documents.onDidClose(change => {
	connection.console.log("Server closed file: " + change.document.uri);
	documentSettings.delete(change.document.uri); // remove from settings
	// remove diagnostics for this file
	connection.sendDiagnostics({ uri: change.document.uri, diagnostics: [] });
	// rerun diagnostics for all open files
	validateAllTextDocuments();
});

async function getAllCompletions(): Promise<CompletionItem[]> {
	const mm = await rust;
	try {
		const result = mm.riscv_get_uncond_completions() as CompletionItem[];
		return result;
	} catch {
		connection.console.log("Server error");
		return [];
	}
}


async function validateAllTextDocuments(): Promise<void> {

	// get all open files
	const all = documents.all();

	// new type for document
	type RVDocument = {
		uri: string,
		text: string,
	};

	type RVDiagnostic = {
		uri: string,
		diagnostics: Diagnostic[],
	};

	// map each file to new doc type
	const rvDocuments: RVDocument[] = [];
	for (const doc of all) {
		rvDocuments.push({ uri: doc.uri, text: doc.getText() });
	}

	// get diagnostics
	const mm = await rust;
	try {
		const result = mm.riscv_get_diagnostics(rvDocuments) as RVDiagnostic[];
		for (const diag of result) {
			connection.sendDiagnostics(diag);
		}
	}
	catch {
		connection.console.log("Server error");
	}


}


// async function validateTextDocument(textDocument: TextDocument): Promise<void> {
// 	// In this simple example we get the settings for every validate run.
// 	//const settings = await getDocumentSettings(textDocument.uri);
// 	const text = textDocument.getText();
// 	const fileuri = textDocument.uri;


// 	let getGetTextContentByUri = (uri: string, imported_by: string | null): string | null => {

// 		if (imported_by) {
// 			// add to imported files for that file
// 			let imported_files = openAndImportedFiles.get(imported_by);
// 			if (imported_files) {
// 				if (imported_files.includes(uri)) {
// 					// we already imported this file, so we don't want to import it again
// 					return null;
// 				} else {
// 					imported_files.push(uri);
// 				}
// 			} 
// 		}

// 		let doc = documents.get(uri);
// 		if (doc) {
// 			return doc.getText();
// 		} else {
// 			return null;
// 		}
// 	};

// 	const mm = await rust;
// 	try {

// 		// for (let item of openAndImportedFiles) {
// 		// 	if (item[1].includes(fileuri)) {
// 		// 		// we don't want to import the file that imports this file
// 		// 		return;
// 		// 	}
// 		// }


// 		const result = mm.riscv_get_diagnostics(text, fileuri, workspaceFolders) as Diagnostic[];

// 		// look through all open files and check if there is any path to the file
// 		// if not, then we don't send diagnostics
// 		// we need to check recursively, because we might have a file that is imported
// 		// by another file, which is imported by another file, etc.
// 		// we also need to keep track of the "visited files"
// 		// so we don't get stuck in a loop






// 		// connection.console.log("Server result: " + JSON.stringify(result));
// 		connection.sendDiagnostics({ uri: textDocument.uri, diagnostics: result });
// 	} catch  {
// 		connection.console.log("Server error");
// 	}
// 	// Send the computed diagnostics to VSCode.
// }

// connection.onDidChangeWatchedFiles(_change => {
// 	// Monitored files have change in VSCode
// 	connection.console.log('We received an file change event');
// });

connection.onCompletion(async (textDocumentPosition: TextDocumentPositionParams): Promise<CompletionItem[]> => {
	const items = await getAllCompletions();
	return items;
});

connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
	return item;
});

// // This handler provides the initial list of the completion items.
// connection.onCompletion(
// 	(_textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
// 		// The pass parameter contains the position of the text document in
// 		// which code complete got requested. For the example we ignore this
// 		// info and always provide the same completion items.
// 		return [
// 			{
// 				label: 'my_module',
// 				kind: CompletionItemKind.Function,
// 				data: 3
// 			},
// 			{
// 				label: 'TypeScript',
// 				kind: CompletionItemKind.Text,
// 				data: 1
// 			},
// 			{
// 				label: 'JavaScript',
// 				kind: CompletionItemKind.Text,
// 				data: 2
// 			}
// 		];
// 	}
// );

// // This handler resolves additional information for the item selected in
// // the completion list.
// connection.onCompletionResolve(
// 	(item: CompletionItem): CompletionItem => {
// 		if (item.data === 1) {
// 			item.detail = 'TypeScript details';
// 			item.documentation = 'TypeScript documentation';
// 		} else if (item.data === 2) {
// 			item.detail = 'JavaScript details';
// 			item.documentation = 'JavaScript documentation';
// 		}
// 		return item;
// 	}
// );

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();
