<script lang="ts">
import { greet } from "./greet";

let { name }: { name: string } = $props();
</script>

<p>{greet(name)}</p>
