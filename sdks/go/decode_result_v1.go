package orchestrata

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"unicode/utf8"
)

// DecodeResultV1 parses a strict ABI v1 result document without converting an
// integer to floating point. A nil solution is returned only for explicit null.
func DecodeResultV1(response []byte) (*Solution, error) {
	// Native views exclude the trailing NUL; the ABI cap includes it.
	if len(response) >= maxV1ResultBytes {
		return nil, fmt.Errorf("result exceeds the ABI v1 byte limit")
	}
	if !utf8.Valid(response) {
		return nil, fmt.Errorf("result must be valid UTF-8")
	}
	document, err := exactObject(response, "solution")
	if err != nil {
		return nil, err
	}
	if bytes.Equal(document["solution"], []byte("null")) {
		return nil, nil
	}
	fields, err := exactObject(document["solution"], "values", "score", "evaluated")
	if err != nil {
		return nil, err
	}
	score, err := decodeScore(fields["score"])
	if err != nil {
		return nil, err
	}
	evaluated, err := exactValue[uint64](fields["evaluated"])
	if err != nil || evaluated == 0 {
		return nil, fmt.Errorf("evaluated must be a positive unsigned 64-bit integer")
	}
	entries, err := exactValue[[]*int64](fields["values"])
	if err != nil {
		return nil, fmt.Errorf("values must be a signed 64-bit integer array: %w", err)
	}
	values := make([]int64, len(entries))
	for index, entry := range entries {
		if entry == nil {
			return nil, fmt.Errorf("values[%d] must not be null", index)
		}
		values[index] = *entry
	}
	return &Solution{Values: values, Score: score, Evaluated: evaluated}, nil
}

func decodeScore(data []byte) (Score, error) {
	fields, err := exactObject(data, "hard", "soft")
	if err != nil {
		return Score{}, err
	}
	hard, err := exactValue[int64](fields["hard"])
	if err != nil || hard != 0 {
		return Score{}, fmt.Errorf("a verified solution must have a zero hard score")
	}
	soft, err := exactValue[int64](fields["soft"])
	if err != nil || soft > 0 {
		return Score{}, fmt.Errorf("soft score must be a nonpositive signed 64-bit integer")
	}
	return Score{Hard: hard, Soft: soft}, nil
}

func exactValue[T any](data []byte) (T, error) {
	var value *T
	var zero T
	if err := json.Unmarshal(data, &value); err != nil {
		return zero, err
	}
	if value == nil {
		return zero, fmt.Errorf("unexpected null")
	}
	return *value, nil
}

func exactObject(data []byte, keys ...string) (map[string]json.RawMessage, error) {
	decoder := json.NewDecoder(bytes.NewReader(data))
	start, err := decoder.Token()
	if err != nil || start != json.Delim('{') {
		return nil, fmt.Errorf("expected JSON object")
	}
	values := make(map[string]json.RawMessage, len(keys))
	for decoder.More() {
		token, err := decoder.Token()
		key, ok := token.(string)
		if err != nil || !ok || !containsKey(keys, key) || values[key] != nil {
			return nil, fmt.Errorf("unknown, duplicate, or invalid JSON object key")
		}
		var raw json.RawMessage
		if err := decoder.Decode(&raw); err != nil {
			return nil, err
		}
		values[key] = raw
	}
	end, err := decoder.Token()
	if err != nil || end != json.Delim('}') || len(values) != len(keys) {
		return nil, fmt.Errorf("expected all required JSON object fields")
	}
	if _, err := decoder.Token(); err != io.EOF {
		return nil, fmt.Errorf("unexpected data after JSON object")
	}
	return values, nil
}

func containsKey(keys []string, key string) bool {
	for _, candidate := range keys {
		if key == candidate {
			return true
		}
	}
	return false
}
