<template>
  <div>
    <Header />
    <b-container>
      <b-row >
        <b-col>
          <h1>Settings</h1>
          <hr>
          <h5><b>API Key:</b> {{ user_data.api_key }}</h5>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<style>
</style>

<script>
import Header from "@/components/headers/Dashboard";
import axios from "axios";

export default {
  name: "Settings",
  components: {
    Header
  },
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
