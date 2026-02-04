import { redirect } from "react-router";
import type { Route } from "./+types/home";
import { checkMembersEmpty } from "../api";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "HomeDrive" },
    { name: "description", content: "Your personal cloud storage" },
  ];
}

export async function loader() {
  try {
    const response = await checkMembersEmpty();
    if (response.is_empty) {
      return redirect("/setup");
    }
    return null;
  } catch (error) {
    console.error("Failed to check members:", error);
    return null;
  }
}

export default function Home() {
  return (
    <main className="flex items-center justify-center pt-16 pb-4">
      <div className="flex-1 flex flex-col items-center gap-16 min-h-0">
        <h1 className="text-2xl font-bold">Welcome to HomeDrive</h1>
        <p className="text-default-500">
          Please set up your administrator account first.
        </p>
      </div>
    </main>
  );
}
