<template>
  <b-container fluid class="d-flex flex-column flex-grow-1">
    <b-row class="justify-content-center text-center">
      <b-col lg="2" md="6" sm="8" xs="12">
        <br /><br />
        <icon-logo height="5em" width="5em" :show_text="false" />
        <h1>New Sybl Account</h1>
        <b-form class="mt-5 mb-3" @submit.prevent="onSubmit">
          <b-form-input
            class="mb-3"
            type="text"
            required
            placeholder="First Name"
            v-model="firstName"
          />
          <b-form-input
            class="mb-3"
            type="text"
            required
            placeholder="Last Name"
            v-model="lastName"
          />
          <b-form-input
            class="mb-3"
            type="text"
            required
            placeholder="Enter Email"
            v-model="email"
          />
          <b-form-input
            type="password"
            required
            placeholder="Password"
            class="mb-3"
            v-model="password"
          />
          <b-form-input
            type="password"
            required
            placeholder="Confirm Password"
            class="mb-3"
            v-model="confirmPassword"
          />
          <b-form-checkbox value="me" class="mb-3" v-model="overAge"
            >I am Over 13</b-form-checkbox
          >
          <b-button
            variant="primary"
            type="submit"
            block
            :disabled="!validCredentials"
          >
            SIGN UP
          </b-button>
        </b-form>
        <p v-show="!validRegistration">
          Something is wrong with your information
        </p>
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center" align-v="end">
      <b-col>
        <p>
          Did you know? You can register as a client and provide models through
          account settings
        </p>
      </b-col>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import axios from "axios";
export default {
  data() {
    return {
      email: "",
      password: "",
      confirmPassword: "",
      overAge: false,
      firstName: "",
      lastName: "",

      validRegistration: true,
    };
  },
  components: {
    IconLogo,
  },
  computed: {
    validCredentials() {
      return (
        this.email &&
        this.firstName &&
        this.lastName &&
        this.password &&
        this.confirmPassword &&
        this.overAge &&
        this.password === this.confirmPassword
      );
    },
  },
  methods: {
    async onSubmit() {
      let response = await axios.post("http://localhost:3001/api/users/new", {
        email: this.email,
        password: this.password,
        firstName: this.firstName,
        lastName: this.lastName,
      });

      response = response.data;

      if (response.token === "null") {
        this.validRegistration = false;
      } else {
        $cookies.set("token", response.token, { path: "/", sameSite: true });

        // Send the user's JWT token on every request type
        axios.defaults.headers.common["Authorization"] = `Bearer ${response.token}`;

        this.$router.push("dashboard");
      }
    },
  },
};
</script>
