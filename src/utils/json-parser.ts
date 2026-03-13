// Dracolich v12 JSON Parsing Utilities

export function parseGovernanceJson<T extends object>(
  output: string,
  agentName: string,
  fallback: T,
  requiredFields: (keyof T)[]
): T {
  try {
    // Extract candidate JSON (code block or raw)
    const match = output.match(/```(?:json)?\s*([\s\S]*?)```/);
    let jsonStr = match?.[1]?.trim();

    if (!jsonStr) {
      const start = output.indexOf('{');
      const end = output.lastIndexOf('}');
      if (start === -1 || end === -1 || start >= end) {
        throw new Error('No JSON structure found');
      }
      jsonStr = output.slice(start, end + 1);
    }

    const parsed = JSON.parse(jsonStr) as T;

    // Validate required fields exist and are non-null
    for (const field of requiredFields) {
      const value = parsed[field];
      if (value === undefined || value === null) {
        throw new Error(`Missing or null field: ${String(field)}`);
      }
    }

    return parsed;
  } catch (error) {
    console.error(`[${agentName}] Parse failed: ${error}`);
    console.debug(`[${agentName}] Raw output (first 500): ${output.substring(0, 500)}`);
    return fallback;
  }
}
