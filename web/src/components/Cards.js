import React from "react";
import { Card } from "react-bootstrap";
import styled from "styled-components";

export const TextCard = styled(Card)` 
    background-color: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
  }
`;

export const ItemCardTemplate = styled(Card)` 
    background-color: ${({ theme }) => theme.body};
    color: ${({ theme }) => theme.text};
    width: 100%;
    margin-top: 0.5rem;
    margin-bottom: 0;
    padding: 0.5rem;
    text-align: left;
    box-shadow: 0 2px 2px 0 rgba(0,0,0,0.1);
  }
`;

export const ProjectCard = ({ title, time }) => {
  return (
    <ItemCardTemplate>
      <h5>{title}</h5>
      <h6>Time Elapsed: {time}</h6>
    </ItemCardTemplate>
  );
};

export const ModelCard = ({ title }) => {
  return (
    <ItemCardTemplate>
      <h5>{title}</h5>
    </ItemCardTemplate>
  );
};
