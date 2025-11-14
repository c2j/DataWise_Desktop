import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface QueryResult {
  row_count: number;
  column_count: number;
  preview: string;
}

interface OperationResult {
  success: boolean;
  message: string;
}

type TabType = "query" | "import" | "export";

function App() {
  const [activeTab, setActiveTab] = useState<TabType>("query");

  // Query tab state
  const [sql, setSql] = useState("SELECT 1 as num");
  const [result, setResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Import tab state
  const [importPath, setImportPath] = useState("");
  const [importFormat, setImportFormat] = useState("csv");
  const [importTableName, setImportTableName] = useState("");
  const [importLoading, setImportLoading] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [importSuccess, setImportSuccess] = useState<string | null>(null);

  // Export tab state
  const [exportSource, setExportSource] = useState("");
  const [exportPath, setExportPath] = useState("");
  const [exportFormat, setExportFormat] = useState("csv");
  const [exportLoading, setExportLoading] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);

  async function executeSql() {
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const queryResult: QueryResult = await invoke("execute_sql", { sql });
      setResult(queryResult);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleImport() {
    setImportLoading(true);
    setImportError(null);
    setImportSuccess(null);

    try {
      const result: OperationResult = await invoke("import_file", {
        path: importPath,
        format: importFormat,
        table_name: importTableName || null,
      });

      if (result.success) {
        setImportSuccess(result.message);
        setImportPath("");
        setImportTableName("");
      } else {
        setImportError(result.message);
      }
    } catch (err) {
      setImportError(String(err));
    } finally {
      setImportLoading(false);
    }
  }

  async function handleExport() {
    setExportLoading(true);
    setExportError(null);
    setExportSuccess(null);

    try {
      const result: OperationResult = await invoke("export_file", {
        source: exportSource,
        path: exportPath,
        format: exportFormat,
      });

      if (result.success) {
        setExportSuccess(result.message);
        setExportPath("");
      } else {
        setExportError(result.message);
      }
    } catch (err) {
      setExportError(String(err));
    } finally {
      setExportLoading(false);
    }
  }

  const parsePreview = () => {
    if (!result) return [];
    try {
      return JSON.parse(result.preview);
    } catch {
      return [];
    }
  };

  const previewData = parsePreview();

  return (
    <main className="container">
      <h1>DataWise - Data Analysis Tool</h1>

      <div className="tabs">
        <button
          className={`tab-button ${activeTab === "query" ? "active" : ""}`}
          onClick={() => setActiveTab("query")}
        >
          SQL Query
        </button>
        <button
          className={`tab-button ${activeTab === "import" ? "active" : ""}`}
          onClick={() => setActiveTab("import")}
        >
          Import
        </button>
        <button
          className={`tab-button ${activeTab === "export" ? "active" : ""}`}
          onClick={() => setActiveTab("export")}
        >
          Export
        </button>
      </div>

      {activeTab === "query" && (
        <div className="editor-section">
          <div className="input-area">
            <label htmlFor="sql-input">SQL Query:</label>
            <textarea
              id="sql-input"
              value={sql}
              onChange={(e) => setSql(e.currentTarget.value)}
              placeholder="Enter your SQL query..."
              rows={6}
            />
            <button onClick={executeSql} disabled={loading}>
              {loading ? "Running..." : "Run Query"}
            </button>
          </div>

          <div className="result-area">
            <h2>Results</h2>
            {error && (
              <div className="error-message">
                <strong>Error:</strong> {error}
              </div>
            )}

            {result && !error && (
              <div className="result-info">
                <p>
                  <strong>Rows:</strong> {result.row_count} | <strong>Columns:</strong>{" "}
                  {result.column_count}
                </p>
              </div>
            )}

            {previewData.length > 0 && (
              <div className="result-table">
                <table>
                  <thead>
                    <tr>
                      {Object.keys(previewData[0]).map((key) => (
                        <th key={key}>{key}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {previewData.map((row: any, idx: number) => (
                      <tr key={idx}>
                        {Object.values(row).map((val: any, colIdx: number) => (
                          <td key={colIdx}>
                            {val === null ? <em>null</em> : String(val)}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {result && previewData.length === 0 && !error && (
              <p className="no-results">No results to display</p>
            )}
          </div>
        </div>
      )}

      {activeTab === "import" && (
        <div className="operation-section">
          <h2>Import File</h2>
          <div className="form-group">
            <label htmlFor="import-path">File Path:</label>
            <input
              id="import-path"
              type="text"
              value={importPath}
              onChange={(e) => setImportPath(e.target.value)}
              placeholder="/path/to/file.csv"
            />
          </div>

          <div className="form-group">
            <label htmlFor="import-format">Format:</label>
            <select
              id="import-format"
              value={importFormat}
              onChange={(e) => setImportFormat(e.target.value)}
            >
              <option value="csv">CSV</option>
              <option value="parquet">Parquet</option>
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="import-table">Table Name (optional):</label>
            <input
              id="import-table"
              type="text"
              value={importTableName}
              onChange={(e) => setImportTableName(e.target.value)}
              placeholder="table_name"
            />
          </div>

          <button onClick={handleImport} disabled={importLoading || !importPath}>
            {importLoading ? "Importing..." : "Import"}
          </button>

          {importError && (
            <div className="error-message">
              <strong>Error:</strong> {importError}
            </div>
          )}

          {importSuccess && (
            <div className="success-message">
              <strong>Success:</strong> {importSuccess}
            </div>
          )}
        </div>
      )}

      {activeTab === "export" && (
        <div className="operation-section">
          <h2>Export Data</h2>
          <div className="form-group">
            <label htmlFor="export-source">Source (Table or SQL):</label>
            <input
              id="export-source"
              type="text"
              value={exportSource}
              onChange={(e) => setExportSource(e.target.value)}
              placeholder="table_name or SELECT * FROM table"
            />
          </div>

          <div className="form-group">
            <label htmlFor="export-path">Export Path:</label>
            <input
              id="export-path"
              type="text"
              value={exportPath}
              onChange={(e) => setExportPath(e.target.value)}
              placeholder="/path/to/export.csv"
            />
          </div>

          <div className="form-group">
            <label htmlFor="export-format">Format:</label>
            <select
              id="export-format"
              value={exportFormat}
              onChange={(e) => setExportFormat(e.target.value)}
            >
              <option value="csv">CSV</option>
              <option value="parquet">Parquet</option>
            </select>
          </div>

          <button onClick={handleExport} disabled={exportLoading || !exportSource || !exportPath}>
            {exportLoading ? "Exporting..." : "Export"}
          </button>

          {exportError && (
            <div className="error-message">
              <strong>Error:</strong> {exportError}
            </div>
          )}

          {exportSuccess && (
            <div className="success-message">
              <strong>Success:</strong> {exportSuccess}
            </div>
          )}
        </div>
      )}
    </main>
  );
}

export default App;
