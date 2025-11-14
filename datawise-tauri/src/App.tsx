import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface QueryResult {
  row_count: number;
  column_count: number;
  preview: string;
}

function App() {
  const [sql, setSql] = useState("SELECT 1 as num");
  const [result, setResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      <h1>DataWise SQL Editor</h1>

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
    </main>
  );
}

export default App;
