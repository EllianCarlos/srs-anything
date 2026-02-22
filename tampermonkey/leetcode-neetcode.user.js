// ==UserScript==
// @name         SRS Anything LeetCode + NeetCode
// @namespace    https://srs-anything.local
// @version      0.1.0
// @description  Capture problem status changes and send to SRS backend
// @author       srs-anything
// @match        http://localhost:5173/*
// @match        https://app.example.com/*
// @match        https://leetcode.com/problems/*
// @match        https://neetcode.io/problems/*
// @grant        GM_xmlhttpRequest
// @grant        GM_getValue
// @grant        GM_setValue
// @connect      localhost
// ==/UserScript==

(function () {
  "use strict";

  const API_BASE = window.localStorage.getItem("srs_api_base") || "http://localhost:3000";
  const TRUSTED_APP_ORIGINS = (window.localStorage.getItem("srs_trusted_app_origins")
    || "http://localhost:5173,https://app.example.com")
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);

  const readApiToken = () => GM_getValue("srs_api_token", "");
  const currentOrigin = window.location.origin;

  if (TRUSTED_APP_ORIGINS.includes(currentOrigin)) {
    window.addEventListener("message", (event) => {
      if (!TRUSTED_APP_ORIGINS.includes(event.origin)) return;
      if (event.data?.source !== "srs-anything") return;
      if (event.data?.type !== "SRS_API_TOKEN_CREATED") return;
      if (typeof event.data?.token !== "string" || event.data.token.length < 20) return;
      GM_setValue("srs_api_token", event.data.token);
      console.log("[srs-anything] API token saved in userscript storage");
    });
    return;
  }

  const getSource = () => (window.location.hostname.includes("leetcode") ? "leetcode" : "neetcode");
  const getSlug = () => window.location.pathname.split("/").filter(Boolean).pop() || "unknown-problem";
  const getTitle = () => document.title.replace(/ - LeetCode| - NeetCode/g, "").trim();
  const getStatus = () => {
    const solvedHint = document.body.innerText.toLowerCase();
    return solvedHint.includes("accepted") || solvedHint.includes("success")
      ? "solved"
      : "unsolved";
  };

  const sendEvent = () => {
    const apiToken = readApiToken();
    if (!apiToken) {
      console.warn("[srs-anything] missing srs_api_token in userscript storage");
      return;
    }

    const payload = {
      source: getSource(),
      problem_slug: getSlug(),
      title: getTitle(),
      url: window.location.href,
      status: getStatus(),
      occurred_at: new Date().toISOString(),
    };

    GM_xmlhttpRequest({
      method: "POST",
      url: `${API_BASE}/events/problem-status`,
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": apiToken,
      },
      data: JSON.stringify(payload),
      onload: (response) => {
        console.log("[srs-anything] ingested event", response.status, payload);
      },
      onerror: (error) => {
        console.error("[srs-anything] ingestion failed", error);
      },
    });
  };

  // Keep this simple for MVP: when page has been stable for a few seconds, emit one event.
  window.setTimeout(sendEvent, 4000);
})();
