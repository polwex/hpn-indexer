import React, { useState, useEffect } from "react";
import { fetchAll, fetchCategory, searchDB } from "../logic/calls";
import { AllProviders, Provider } from "../logic/types";

const Main: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [searchResults, setSearchResults] = useState<Provider[]>([]);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [input, setInput] = useState("");
  // const [data, setData] = useState<AllProviders>({});
  const [data, setData] = useState<Provider[]>([]);

  // Import our.js from the host URL
  useEffect(() => {
    const script = document.createElement("script");
    script.src = window.location.origin + "/our.js";
    document.head.appendChild(script);
  }, []);

  useEffect;
  // useEffect(() => {
  //   fetchAll().then((data) => {
  //     if ("error" in data) setSearchError("error fetching data");
  //     else setData(data.ok);
  //   });
  // }, []);

  // function handleSearch() {
  //   setSearchResults([]);
  //   setSearchError("");
  //   console.log("lol", input);
  //   const inp = input.toLowerCase();
  //   for (let provlist of Object.values(data)) {
  //     for (let prov of provlist) {
  //       if (
  //         prov.category.toLowerCase().includes(inp) ||
  //         prov.description.toLowerCase().includes(inp) ||
  //         prov.name.toLowerCase().includes(inp) ||
  //         prov.providerName.toLowerCase().includes(inp) ||
  //         prov.site.toLowerCase().includes(inp)
  //       )
  //         setSearchResults((s) => [...s, prov]);
  //     }
  //   }
  //   console.log(searchResults.length, "length");
  //   if (searchResults.length === 0) setSearchError("No providers found");
  // }
  async function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    setSearchResults([]);
    setSearchError("");
    const inp = input.toLowerCase();
    setLoading(true);
    const res = await searchDB(inp);
    console.log({ res });
    if ("error" in res) setSearchError("error searching index");
    else {
      setSearchResults(res.ok);
      if (res.ok.length === 0) setSearchError("No providers found");
    }
    setLoading(false);
  }

  return (
    <div className="explorer-container">
      <div className="search-container">
        {searchError && <div className="search-error">{searchError}</div>}
        <form onSubmit={handleSearch}>
          <input
            type="text"
            name="search"
            placeholder="Steam"
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              setSearchError("");
            }}
            className="search-input"
          />
          <button type="submit" className="search-button">
            Search
          </button>
        </form>
      </div>
      {loading ? (
        <p>Searching...</p>
      ) : (
        searchResults.length > 0 && <SearchResults results={searchResults} />
      )}
    </div>
  );
};

export default Main;
// TODO we might want to allow icons or images for the providers and more cosmetic stuff
function SearchResults({ results }: { results: Provider[] }) {
  return (
    <div>
      {results.map((r) => (
        <div className="search-result" key={r.name}>
          <h3>{r.name}</h3>
          <p>Provider name: {r.providerName}</p>
          <p>Description: {r.description}</p>
          <p>Category: {r.category}</p>

          <a href={r.site}>Go to site</a>
        </div>
      ))}
    </div>
  );
}
