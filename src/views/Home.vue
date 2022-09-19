<template>
  <div class="home">
    <img alt="Vue logo" src="../assets/logo.png" />
    <!-- <HelloWorld msg="Welcome to Your Vue.js App" /> -->
    <button @click="connect">Connect</button>
  </div>
</template>

<script>
// @ is an alias to /src
// import HelloWorld from "@/components/HelloWorld.vue";

export default {
  name: "Home",
  // components: {
  //   HelloWorld
  // },
  methods: {
    async connect() {
      console.log("connect");
      console.log(this.$wasm);

      const client = await this.$wasm.start_client(
        "/ip4/127.0.0.1/tcp/38615/ws",
        "debug"
      );
      console.log("rust libp2p client:", client);
    }
  }
};
</script>
