import { assertEquals } from "https://deno.land/std/testing/asserts.ts";
import { Patch, patch } from "../util.ts";

const patches = [
  new Patch(8, 9, "b"),
  new Patch(4, 5, "b"),
  new Patch(3, 7, "a"),
];

Deno.test("patching", () => {
  assertEquals(patch("0123456789", patches), "012a7b9");
});
