<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Settings</h1>
        <hr>
        <h5><b>API Key:</b> {{ user_data.api_key }}</h5>
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
</style>
<script>
import axios from "axios";

export default {
  name: "Settings",
  data() {
    return {
      user_data: {},
    }
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let data = await axios.get(
        `http://localhost:3001/api/users/${user_id}`
      );
      this.user_data = data.data
    } catch (err) {
      console.log(err);
    }
      console.log(this.user_data)
  },
  methods: {},
};
</script>
